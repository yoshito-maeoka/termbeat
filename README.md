# 🎵 Rust Rhythm Box


 ████████╗ ███████╗ ██████╗  ███╗   ███╗ ██████╗  ███████╗  █████╗  ████████╗
 ╚══██╔══╝ ██╔════╝ ██╔══██╗ ████╗ ████║ ██╔══██╗ ██╔════╝ ██╔══██╗ ╚══██╔══╝
    ██║    █████╗   ██████╔╝ ██╔████╔██║ ██████╔╝ █████╗   ███████║    ██║   
    ██║    ██╔══╝   ██╔══██╗ ██║╚██╔╝██║ ██╔══██╗ ██╔══╝   ██╔══██║    ██║   
    ██║    ███████╗ ██║  ██║ ██║ ╚═╝ ██║ ██████╔╝ ███████╗ ██║  ██║    ██║   


*** a spontaneous project driven by Claude ***

A terminal-based rhythm sequencer and drum machine built with Rust. Create beats, compose patterns, and export your music directly from your terminal!

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Terminal](https://img.shields.io/badge/Terminal-UI-blue?style=for-the-badge)

## ✨ Features

- **Terminal User Interface (TUI)**: Beautiful and responsive interface built with `ratatui`
- **4-Track Step Sequencer**: 16-step patterns for Kick, Snare, Hi-Hat, and Bass
- **Real-time Audio Playback**: Low-latency audio output using `cpal`
- **Synthesized Instruments**: 
  - **Kick**: Deep bass drum with pitch envelope
  - **Snare**: Noise + tone synthesis for realistic snare sound
  - **Hi-Hat**: High-frequency noise bursts
  - **Bass**: Sine wave bass synthesizer
- **WAV Export**: Export your patterns to 44.1kHz/16-bit WAV files
- **Adjustable BPM**: Currently set to 120 BPM (customizable in code)


## 🚀 Getting Started

### Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)

### Installation

```bash
# Clone or create the project
cargo new rhythm-box
cd rhythm-box

# Add dependencies
cargo add ratatui crossterm cpal hound
```

### Running

```bash
cargo run
```

## 🎹 Controls

### Navigation
- **Arrow Keys (← → ↑ ↓)**: Move cursor between steps and tracks
- **Space**: Toggle step on/off
- **Enter**: Start/Stop playback

### Export
- **E**: Export pattern to WAV file (4 loops)
  - Files are saved as `rhythm-box-[timestamp].wav`
  - Saved in the project root directory

### General
- **Q**: Quit application

## 🎼 How to Use

1. **Create a Pattern**:
   - Use arrow keys to navigate the 16-step grid
   - Press Space to activate/deactivate steps
   - Active steps are shown as `●`, inactive as `○`

2. **Play Your Beat**:
   - Press Enter to start playback
   - Watch the green highlight move across your pattern
   - Press Enter again to stop

3. **Export to WAV**:
   - Press E to export your pattern
   - The file will be saved in the project directory
   - Share it with friends or import into your DAW!

## 🎨 Example Patterns

### Basic 4/4 Beat
```
Kick:   ●  ○  ○  ○  ●  ○  ○  ○  ●  ○  ○  ○  ●  ○  ○  ○
Snare:  ○  ○  ○  ○  ●  ○  ○  ○  ○  ○  ○  ○  ●  ○  ○  ○
Hi-Hat: ●  ○  ●  ○  ●  ○  ●  ○  ●  ○  ●  ○  ●  ○  ●  ○
Bass:   ●  ○  ○  ○  ○  ○  ○  ○  ●  ○  ○  ○  ○  ○  ○  ○
```

### Dense Techno
```
Kick:   ●  ○  ○  ○  ●  ○  ○  ○  ●  ○  ○  ○  ●  ○  ○  ○
Snare:  ○  ○  ○  ○  ●  ○  ○  ○  ○  ○  ○  ○  ●  ○  ○  ○
Hi-Hat: ●  ●  ●  ●  ●  ●  ●  ●  ●  ●  ●  ●  ●  ●  ●  ●
Bass:   ●  ○  ●  ○  ○  ○  ●  ○  ●  ○  ○  ○  ●  ○  ○  ○
```

## 🛠️ Technical Details

### Architecture
- **TUI Thread**: Handles user input and rendering (main thread)
- **Audio Thread**: Generates and outputs audio samples in real-time
- **Communication**: Lockless communication via `Arc<Mutex<>>` for triggered sounds

### Audio Specifications
- **Sample Rate**: 44.1 kHz
- **Bit Depth**: 16-bit (WAV export)
- **Channels**: Mono
- **Latency**: Optimized for real-time playback

### Dependencies
```toml
[dependencies]
ratatui = "0.26"    # Terminal UI framework
crossterm = "0.27"  # Terminal control
cpal = "0.15"       # Cross-platform audio
hound = "3.5"       # WAV file I/O
```

## 🔧 Customization

### Changing BPM
Modify the `bpm` field in the `App::new()` function:
```rust
bpm: 140,  // Change from default 120
```

### Adding More Steps
Change the pattern length:
```rust
steps: vec![false; 32],  // 32 steps instead of 16
length: 32,
```

### Adjusting Instrument Sounds
Modify the synthesis parameters in `AudioEngine`:
- Kick frequency, decay time
- Snare noise/tone balance
- Hi-Hat duration
- Bass note values

## 🎯 Future Enhancements

Potential features to add:
- [ ] BPM adjustment from UI
- [ ] Multiple pattern banks
- [ ] Save/Load patterns to file
- [ ] More instrument types
- [ ] More preset Sounds
- [ ] Effects (reverb, delay, filters)
- [ ] Longer sequences (32, 64 steps)
- [ ] Velocity per step
- [ ] Swing/groove settings
- [ ] better controllable TUI

## 📝 License

This project is open source and available under the MIT License.

## 🤝 Contributing

Contributions are welcome! Feel free to:
- Report bugs
- Suggest new features
- Submit pull requests
- Share your beats!

## 🙏 Acknowledgments

Built with:
- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [cpal](https://github.com/RustAudio/cpal) - Cross-platform audio library
- [hound](https://github.com/ruuda/hound) - WAV encoding/decoding

---

**Made with ❤️ and Rust**

Enjoy making beats in your terminal! 🎶