use midir::{MidiOutput, MidiOutputConnection};
use anyhow::{Result, anyhow};

pub struct MidiDevice {
    pub connection: MidiOutputConnection,
}

pub fn list_devices() -> Result<()> {
    let midi_out = MidiOutput::new("MIDI Van")?;
    let ports = midi_out.ports();

    if ports.is_empty() {
        println!("No MIDI output devices found.");
        println!("Make sure your USB MIDI cable is connected.");
        return Ok(());
    }

    println!("Available MIDI output devices:");
    for (i, port) in ports.iter().enumerate() {
        if let Ok(name) = midi_out.port_name(port) {
            println!("  {}: {}", i, name);
        }
    }

    Ok(())
}

pub fn get_device(device_name: Option<&String>) -> Result<MidiDevice> {
    let midi_out = MidiOutput::new("MIDI Van")?;
    let ports = midi_out.ports();

    if ports.is_empty() {
        return Err(anyhow!("No MIDI output devices found. Make sure your USB MIDI cable is connected."));
    }

    let selected_port = match device_name {
        Some(name) => {
            ports
                .iter()
                .find(|port| {
                    midi_out.port_name(port)
                        .map(|port_name| port_name.contains(name))
                        .unwrap_or(false)
                })
                .ok_or_else(|| anyhow!("MIDI device '{}' not found", name))?
                .clone()
        }
        None => {
            if ports.len() == 1 {
                ports[0].clone()
            } else {
                println!("Multiple MIDI devices found. Please specify one:");
                for (i, port) in ports.iter().enumerate() {
                    if let Ok(name) = midi_out.port_name(port) {
                        println!("  {}: {}", i, name);
                    }
                }
                return Err(anyhow!("Multiple devices found. Use -d/--device to specify one."));
            }
        }
    };

    let port_name = midi_out.port_name(&selected_port)?;
    println!("Connecting to MIDI device: {}", port_name);

    let connection = midi_out.connect(&selected_port, "midi-van")
        .map_err(|e| anyhow!("Failed to connect to MIDI device: {:?}", e))?;

    Ok(MidiDevice { connection })
}