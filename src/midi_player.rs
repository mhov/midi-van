use crate::device_manager::MidiDevice;
use crate::midi_scraper;
use midly::{Smf, Timing, TrackEventKind, MidiMessage};
use midly::num::u4;
use anyhow::{Result, anyhow};
use rand::seq::SliceRandom;
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub struct MidiPlayer {
    device: MidiDevice,
    tempo: u32,
}

impl MidiPlayer {
    pub fn new(device: MidiDevice) -> Self {
        Self {
            device,
            tempo: 500000, // Default: 120 BPM (500,000 microseconds per quarter note)
        }
    }

    pub async fn play_file(&mut self, file_path: &str) -> Result<()> {
        println!("Playing: {}", file_path);

        let data = std::fs::read(file_path)?;
        let smf = Smf::parse(&data)?;

        let ticks_per_beat = match smf.header.timing {
            Timing::Metrical(tpb) => tpb.as_int() as u32,
            Timing::Timecode(_, _) => return Err(anyhow!("Timecode timing not supported")),
        };

        let mut events = Vec::new();

        for (_track_idx, track) in smf.tracks.iter().enumerate() {
            let mut current_time = 0u32;

            for event in track.iter() {
                current_time += event.delta.as_int();

                match &event.kind {
                    TrackEventKind::Midi { channel, message } => {
                        events.push((current_time, *channel, message.clone()));
                    }
                    TrackEventKind::Meta(meta) => {
                        if let midly::MetaMessage::Tempo(tempo) = meta {
                            self.tempo = tempo.as_int();
                            println!("Tempo change: {} µs per quarter note", self.tempo);
                        }
                    }
                    _ => {}
                }
            }
        }

        events.sort_by_key(|(time, _, _)| *time);

        if events.is_empty() {
            return Err(anyhow!("No MIDI events found in file"));
        }

        let start_time = Instant::now();
        let mut last_time = 0u32;

        for (event_time, channel, message) in events {
            let time_diff = event_time - last_time;
            if time_diff > 0 {
                let duration_ms = (time_diff as f64 * self.tempo as f64) / (ticks_per_beat as f64 * 1000.0);
                sleep(Duration::from_millis(duration_ms as u64)).await;
            }

            if let Err(e) = self.send_midi_message(channel, &message) {
                eprintln!("Error sending MIDI message: {}", e);
            }

            last_time = event_time;
        }

        self.send_all_notes_off()?;

        let duration = start_time.elapsed();
        println!("Playback completed in {:.2} seconds", duration.as_secs_f64());

        Ok(())
    }

    fn send_midi_message(&mut self, channel: u4, message: &MidiMessage) -> Result<()> {
        let mut midi_data = vec![0u8; 3];

        match message {
            MidiMessage::NoteOff { key, vel } => {
                midi_data[0] = 0x80 | channel.as_int();
                midi_data[1] = key.as_int();
                midi_data[2] = vel.as_int();
            }
            MidiMessage::NoteOn { key, vel } => {
                midi_data[0] = 0x90 | channel.as_int();
                midi_data[1] = key.as_int();
                midi_data[2] = vel.as_int();
            }
            MidiMessage::Aftertouch { key, vel } => {
                midi_data[0] = 0xA0 | channel.as_int();
                midi_data[1] = key.as_int();
                midi_data[2] = vel.as_int();
            }
            MidiMessage::Controller { controller, value } => {
                midi_data[0] = 0xB0 | channel.as_int();
                midi_data[1] = controller.as_int();
                midi_data[2] = value.as_int();
            }
            MidiMessage::ProgramChange { program } => {
                midi_data[0] = 0xC0 | channel.as_int();
                midi_data[1] = program.as_int();
                midi_data.truncate(2);
            }
            MidiMessage::ChannelAftertouch { vel } => {
                midi_data[0] = 0xD0 | channel.as_int();
                midi_data[1] = vel.as_int();
                midi_data.truncate(2);
            }
            MidiMessage::PitchBend { bend } => {
                midi_data[0] = 0xE0 | channel.as_int();
                let bend_value = bend.as_int() as u16;
                midi_data[1] = (bend_value & 0x7F) as u8;
                midi_data[2] = ((bend_value >> 7) & 0x7F) as u8;
            }
        }

        self.device.connection.send(&midi_data)?;
        Ok(())
    }

    fn send_all_notes_off(&mut self) -> Result<()> {
        for channel in 0..16 {
            let all_notes_off = [0xB0 | channel, 123, 0];
            self.device.connection.send(&all_notes_off)?;
        }
        Ok(())
    }
}

pub async fn start_random_playback(
    device: MidiDevice,
    midi_urls: Vec<String>,
    cache_dir: &str,
) -> Result<()> {
    if midi_urls.is_empty() {
        return Err(anyhow!("No MIDI URLs available"));
    }

    let mut player = MidiPlayer::new(device);
    let mut rng = rand::thread_rng();

    println!("Starting random playback mode. Press Ctrl+C to exit.");
    println!("Found {} MIDI files to choose from", midi_urls.len());

    loop {
        let selected_url = midi_urls.choose(&mut rng).unwrap();

        match midi_scraper::download_midi_file(selected_url, cache_dir).await {
            Ok(file_path) => {
                if let Err(e) = player.play_file(&file_path).await {
                    eprintln!("Error playing file {}: {}", file_path, e);
                    eprintln!("Trying next file...");
                    continue;
                }

                println!("Waiting 2 seconds before next song...");
                sleep(Duration::from_secs(2)).await;
            }
            Err(e) => {
                eprintln!("Error downloading {}: {}", selected_url, e);
                eprintln!("Trying next file...");
                continue;
            }
        }
    }
}