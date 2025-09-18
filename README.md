# Chroma Tuner

A lightweight, native instrument tuner with real-time pitch detection built in Rust.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey.svg)]()

## Goals

- Accurate real-time pitch detection
- Minimal resource usage
- Native performance on macOS and Linux
- Simple, focused interface

## Features

- Real-time pitch detection using FFT analysis
- Visual tuning display with needle and cent deviation
- Audio device selection
- Stable readings with noise filtering
- Cross-platform support (macOS, Linux)

## Usage

### Building

#### Command Line Binary
```bash
# Build release version
cargo build --release

# Run directly
cargo run --release
```

#### macOS App Bundle
```bash
# Install cargo-bundle (one-time setup)
cargo install cargo-bundle

# Create .app bundle
cargo bundle --release

# Or use the bundling script (recommended)
./scripts/bundle-mac.sh
```

The app bundle will be created at `target/bundle/osx/Chroma Tuner.app` and can be dragged to your Applications folder.

### Tuning
1. Run the application
2. Select your audio input device from the dropdown
3. Play a note on your instrument
4. The display shows:
   - Note name and frequency
   - Tuning needle (centered when in tune)
   - Cent deviation from perfect pitch

### Color coding
- **Green**: In tune (±5 cents)
- **Orange**: Close (±20 cents)  
- **Red**: Out of tune (>20 cents)

## File Structure

- **`src/main.rs`**: Application entry point, window setup, and eframe initialization
- **`src/audio.rs`**: Audio input capture, device management, and sample buffering
- **`src/pitch.rs`**: FFT-based pitch detection and frequency-to-note conversion
- **`src/tuner.rs`**: Main application logic, GUI rendering, and signal filtering

## Distribution

This project uses `cargo-dist` for cross-platform binary distribution and `cargo-bundle` for macOS app bundles:

- **Binaries**: Available via GitHub releases for macOS (Intel/Apple Silicon) and Linux
- **Homebrew**: `brew install cinbarker/tap/chroma-tuner` (coming soon)
- **macOS App**: Download `.app` bundle from releases or build locally

## Technical Details

- **Audio**: `cpal` for cross-platform audio capture
- **DSP**: `realfft` for pitch detection with 8192-sample FFT
- **GUI**: `egui` for native interface
- **Binary size**: ~6MB
- **Frequency range**: 80Hz - 2000Hz

## Prerequisites

### macOS
- Rust toolchain
- Core Audio (included with system)

### Linux
- Rust toolchain
- ALSA development libraries:
  ```bash
  # Ubuntu/Debian
  sudo apt install libasound2-dev
  
  # Fedora/RHEL
  sudo dnf install alsa-lib-devel
  ```
