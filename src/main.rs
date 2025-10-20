use clap::{Arg, Command};
use anyhow::Result;

mod midi_scraper;
mod midi_player;
mod device_manager;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("midi-van")
        .version("0.1.0")
        .author("MIDI Van")
        .about("Downloads and plays random classical piano MIDI files")
        .arg(
            Arg::new("device")
                .short('d')
                .long("device")
                .value_name("DEVICE")
                .help("MIDI output device name (lists available devices if not specified)")
        )
        .arg(
            Arg::new("list-devices")
                .short('l')
                .long("list-devices")
                .action(clap::ArgAction::SetTrue)
                .help("List available MIDI devices and exit")
        )
        .arg(
            Arg::new("cache-dir")
                .short('c')
                .long("cache-dir")
                .value_name("DIR")
                .default_value("./midi_cache")
                .help("Directory to cache downloaded MIDI files")
        )
        .get_matches();

    if matches.get_flag("list-devices") {
        device_manager::list_devices()?;
        return Ok(());
    }

    let cache_dir = matches.get_one::<String>("cache-dir").unwrap();
    let device_name = matches.get_one::<String>("device");

    println!("MIDI Van starting...");

    let device = device_manager::get_device(device_name)?;
    let midi_urls = midi_scraper::fetch_midi_urls().await?;

    midi_player::start_random_playback(device, midi_urls, cache_dir).await?;

    Ok(())
}
