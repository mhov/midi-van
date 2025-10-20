# MIDI Van

A Rust application that downloads random classical piano MIDI files from piano-midi.de and plays them through USB-connected MIDI devices.

## Use Case

I have a Yamaha P71 keyboard that sounds great, and I wanted to explore the possibilities of playing classical piano pieces on it. Like a rust based player piano.

## AI-Assisted Development

Most of this codebase were generated with the assistance of ClaudeCode during development.
I wanted to try out this idea.

## Features

- Downloads and caches MIDI files from piano-midi.de
- Supports all major classical composers (Bach, Mozart, Beethoven, Chopin, etc.)
- Random playback of classical piano pieces
- Cross-platform support (macOS and Raspberry Pi)
- USB MIDI device output
- Command-line interface with device selection

## Requirements

- Rust 1.70+ (2024 edition)
- USB MIDI cable/interface
- MIDI-compatible piano or synthesizer

## Installation

```bash
cargo build --release
```

## Usage

### List available MIDI devices
```bash
./target/release/midi-van --list-devices
```

### Start random playback (auto-select device if only one available)
```bash
./target/release/midi-van
```

### Specify a specific MIDI device
```bash
./target/release/midi-van --device "USB MIDI Interface"
```

### Use custom cache directory
```bash
./target/release/midi-van --cache-dir /path/to/cache
```

## Cross-compilation for Raspberry Pi

### From macOS to Raspberry Pi (ARM64)

1. Install the target:
```bash
rustup target add aarch64-unknown-linux-gnu
```

2. Install cross-compilation tools:
```bash
brew install aarch64-elf-gcc
```

3. Configure Cargo for cross-compilation by creating `.cargo/config.toml`:
```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```

4. Build for Raspberry Pi:
```bash
cargo build --release --target aarch64-unknown-linux-gnu
```

### Alternative: Using cross

Install cross for easier cross-compilation:
```bash
cargo install cross
cross build --release --target aarch64-unknown-linux-gnu
```

## Raspberry Pi Setup

1. Copy the compiled binary to your Raspberry Pi
2. Install required system dependencies:
```bash
sudo apt update
sudo apt install libasound2-dev
```

3. Connect your USB MIDI cable to the Pi
4. Run the application:
```bash
./midi-van --list-devices
./midi-van
```

## Troubleshooting

### No MIDI devices found
- Ensure your USB MIDI cable is properly connected
- On Linux/Raspberry Pi, check that your user is in the `audio` group:
  ```bash
  sudo usermod -a -G audio $USER
  ```
- Verify the device is recognized:
  ```bash
  lsusb  # Should show your MIDI interface
  aconnect -l  # List ALSA MIDI connections
  ```

### Network/Download issues
- Ensure you have internet connectivity
- The application caches downloaded files in `./midi_cache/` by default
- If downloads fail, try again as some files may be temporarily unavailable

### Playback issues
- Verify your MIDI receiving device (piano/synthesizer) is set to receive on the correct channel
- Some MIDI files may have different channel configurations
- Try different MIDI files if one doesn't play correctly

## Architecture

The application consists of several modules:

- `midi_scraper.rs` - Web scraping and MIDI file downloading
- `device_manager.rs` - MIDI device discovery and connection
- `midi_player.rs` - MIDI file parsing and playback with accurate timing
- `main.rs` - CLI interface and application coordination

## Dependencies

- `midir` - Cross-platform MIDI I/O
- `reqwest` - HTTP client for downloading files
- `scraper` - HTML parsing
- `midly` - MIDI file parsing
- `tokio` - Async runtime
- `clap` - Command-line argument parsing
- `rand` - Random selection
