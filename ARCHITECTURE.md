# Codex Voice Architecture & Operations Guide

## 🏗️ System Overview

Codex Voice is a voice-controlled interface for the Codex AI assistant, featuring a Rust TUI (Terminal User Interface) with real-time audio capture, speech-to-text transcription, and AI interaction.

```
┌─────────────────────────────────────────────────────────┐
│                     USER INTERACTION                     │
│                    (Voice + Keyboard)                    │
└────────────────────┬───────────────────────────┬────────┘
                      │                           │
                      ▼                           ▼
┌─────────────────────────────┐   ┌──────────────────────┐
│      RUST TUI (Main)        │   │   Voice Input ('v')   │
│   rust_tui/src/main.rs      │   │                      │
│   - Terminal UI (ratatui)   │◄──┤  Captures audio      │
│   - Event loop              │   │  Transcribes speech  │
│   - Status display          │   │  Sends to Codex     │
└─────────────┬───────────────┘   └──────────────────────┘
              │
              ▼
┌─────────────────────────────────────────────────────────┐
│                    AUDIO PIPELINE                        │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────────────┐     ┌──────────────┐                 │
│  │ RUST NATIVE  │     │   PYTHON     │                 │
│  │   (FAST)     │ OR  │  FALLBACK    │                 │
│  │              │     │   (SLOW)     │                 │
│  └──────────────┘     └──────────────┘                 │
│                                                          │
│  Rust Path:                Python Path:                 │
│  1. audio.rs (cpal)        1. codex_voice.py           │
│  2. Resample to 16kHz      2. pyaudio recording        │
│  3. stt.rs (whisper-rs)    3. whisper transcription    │
│  4. Direct to Codex        4. JSON response            │
│                                                          │
└─────────────────────────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────────────────────┐
│                    CODEX INTEGRATION                     │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────────────────────────────────────┐              │
│  │         PTY Session Manager           │              │
│  │      (pty_session.rs)                │              │
│  │   - Spawns Codex process             │              │
│  │   - Manages I/O streams              │              │
│  │   - Handles terminal queries         │              │
│  └──────────────────────────────────────┘              │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## 📁 Project Structure

```
codex_voice/
│
├── rust_tui/                   # Main Rust TUI application
│   ├── Cargo.toml             # Rust dependencies
│   ├── src/
│   │   ├── main.rs            # Entry point, TUI setup
│   │   ├── app.rs             # Application state & logic
│   │   ├── ui.rs              # Terminal UI rendering
│   │   ├── audio.rs           # Audio recording (cpal)
│   │   ├── stt.rs             # Speech-to-text (whisper-rs)
│   │   ├── voice.rs           # Voice capture orchestration
│   │   ├── pty_session.rs     # PTY/terminal management
│   │   ├── config.rs          # Configuration & CLI args
│   │   └── utf8_safe.rs       # UTF-8 safe string ops
│   │
│   └── target/
│       └── release/
│           └── rust_tui       # Compiled binary
│
├── models/                     # Whisper AI models
│   ├── ggml-tiny.en.bin      # Fastest (74MB)
│   └── ggml-base.en.bin      # Better quality (141MB)
│
├── codex_voice.py             # Python fallback pipeline
├── scripts/                    # Helper scripts
└── ARCHITECTURE.md            # This file

```

## 🚀 Quick Start Commands

### Build & Run
```bash
# Navigate to project
cd /Users/jguida941/new_github_projects/codex_voice/rust_tui

# Build (debug mode - slow but with symbols)
cargo build

# Build (release mode - fast, optimized)
cargo build --release

# Run directly
cargo run --release

# Or run compiled binary
./target/release/rust_tui
```

### Download Whisper Models (Required for Rust path)
```bash
# Tiny model (fastest, 74MB, lower quality)
curl -L "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin" \
     -o models/ggml-tiny.en.bin

# Base model (slower, 141MB, better quality)
curl -L "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin" \
     -o models/ggml-base.en.bin

# Small model (even slower, 488MB, good quality)
curl -L "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin" \
     -o models/ggml-small.en.bin
```

## ⚙️ Configuration Options

### Command Line Arguments
```bash
# Show all options
./target/release/rust_tui --help

# Common options
--seconds 2                    # Recording duration (default: 3)
--lang en                      # Language (default: en)
--input-device "MacBook Pro Microphone"  # Select microphone
--list-input-devices           # List available mics
--log-timings                  # Enable performance logging
--model ../models/ggml-tiny.en.bin  # Specify model path
--codex-cmd /path/to/codex    # Path to Codex binary
--no-python-fallback           # Disable Python fallback
```

### Examples
```bash
# Fast setup (2 sec recording, tiny model)
./target/release/rust_tui --seconds 2 --model ../models/ggml-tiny.en.bin

# High quality (5 sec recording, base model)
./target/release/rust_tui --seconds 5 --model ../models/ggml-base.en.bin

# Debug performance issues
./target/release/rust_tui --log-timings

# Use specific microphone
./target/release/rust_tui --input-device "MacBook Pro Microphone"
```

## 🎤 Voice Pipeline Decision Tree

```
User presses 'v'
    │
    ▼
Is Whisper model available?
    │
    ├─ YES ─→ Use RUST NATIVE path
    │         │
    │         ├─ Record with cpal (audio.rs)
    │         ├─ Resample to 16kHz mono
    │         ├─ Transcribe with whisper-rs (stt.rs)
    │         └─ Send to Codex
    │
    └─ NO ──→ Use PYTHON FALLBACK path
              │
              ├─ Spawn Python subprocess
              ├─ Run codex_voice.py
              ├─ Parse JSON response
              └─ Send to Codex
```

## 🔧 Key Components

### 1. Main Application (`app.rs`)
- Manages application state
- Handles keyboard/voice events
- Coordinates between UI and backend
- UTF-8 safe text processing

### 2. Audio Recording (`audio.rs`)
- Uses `cpal` for cross-platform audio
- Downmixes to mono
- Resamples to 16kHz (Whisper requirement)
- Two resamplers:
  - High-quality: Rubato (feature-gated)
  - Basic: Linear interpolation

### 3. Speech-to-Text (`stt.rs`)
- Uses `whisper-rs` bindings
- Loads GGML models
- Runs inference on CPU
- Returns transcribed text

### 4. Voice Orchestration (`voice.rs`)
- Spawns background worker thread
- Tries Rust path first
- Falls back to Python if needed
- Reports status to UI

### 5. PTY Session (`pty_session.rs`)
- Creates pseudo-terminal
- Spawns Codex process
- Handles terminal control sequences
- Manages I/O streams

### 6. UI Rendering (`ui.rs`)
- Uses `ratatui` for TUI
- Real-time status updates
- UTF-8 safe text display
- No text wrapping (avoids ratatui bug)

## 🐛 Debug & Logs

### Log File Location
```bash
# Debug log (automatic)
/tmp/codex_voice_tui.log

# View live logs
tail -f /tmp/codex_voice_tui.log

# Check for errors
grep -i error /tmp/codex_voice_tui.log

# Check pipeline path
grep -E "(Rust pipeline|Python fallback)" /tmp/codex_voice_tui.log
```

### Performance Analysis
```bash
# Run with timing logs
./target/release/rust_tui --log-timings

# Check timings in log
grep "timing|phase=voice_capture" /tmp/codex_voice_tui.log

# Format: record_s=X.XXX|stt_s=X.XXX|chars=XXX
# record_s = recording duration
# stt_s = transcription time
# chars = transcript length
```

## 🚄 Performance Optimization

### Current Bottlenecks
1. **Whisper inference** (~1-3s on CPU)
2. **Recording duration** (default 3s)
3. **Python subprocess** (if fallback)
4. **Model size** (larger = slower)

### Speed Improvements
```bash
# Use tiny model (fastest)
--model ../models/ggml-tiny.en.bin

# Reduce recording time
--seconds 2

# Ensure release build
cargo build --release

# Use native Rust path (avoid Python)
# (Requires Whisper model in models/)

# Select best audio device
--list-input-devices
--input-device "Your Best Mic"
```

### Build Features
```bash
# With high-quality audio resampling
cargo build --release --features high-quality-audio

# Minimal build (faster compile, basic resampling)
cargo build --release --no-default-features
```

## 🧪 Testing

### Unit Tests
```bash
# Run all tests
cargo test

# With high-quality audio feature
cargo test --features high-quality-audio

# Specific test
cargo test test_name
```

### Integration Test
```bash
# Test audio devices
./target/release/rust_tui --list-input-devices

# Test voice capture (manual)
./target/release/rust_tui
# Press 'v', speak, check status line
```

## 📊 Status Line Indicators

When running the TUI, the bottom status line shows:

- **"Ready"** - Waiting for input
- **"Recording..."** - Capturing audio
- **"Processing..."** - Transcribing speech
- **"Rust pipeline"** - Using native Rust path (fast)
- **"Python fallback"** - Using Python subprocess (slow)
- **Error messages** - If something fails

## 🔑 Key Bindings

- `v` - Start voice capture
- `Enter` - Send text input to Codex
- `Tab` - Cycle between input modes
- `↑/↓` - Scroll through history
- `Ctrl+C` or `q` - Quit

## 🎯 Common Issues & Solutions

### Issue: "Python fallback" (slow)
**Solution:** Download Whisper model
```bash
curl -L "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin" \
     -o models/ggml-tiny.en.bin
```

### Issue: No audio devices found
**Solution:** Check permissions
```bash
# macOS: System Preferences → Security & Privacy → Microphone
# Grant terminal/app microphone access
```

### Issue: Panic with large byte index
**Solution:** Already fixed! Our patches handle:
- UTF-8 safe string slicing
- Saturating arithmetic
- Mutex poisoning recovery
- Terminal query responses

### Issue: Slow transcription
**Solution:** Use tiny model + shorter recording
```bash
./target/release/rust_tui --seconds 2 --model ../models/ggml-tiny.en.bin
```

## 📝 Development Workflow

```bash
# 1. Make code changes
vim src/app.rs

# 2. Check compilation
cargo check

# 3. Run tests
cargo test

# 4. Build release
cargo build --release

# 5. Test manually
./target/release/rust_tui

# 6. Check logs
tail -f /tmp/codex_voice_tui.log
```

## 🔮 Future Improvements

- [ ] GPU acceleration for Whisper
- [ ] Streaming transcription
- [ ] Voice activity detection (VAD)
- [ ] Wake word detection
- [ ] Multi-language support
- [ ] Noise suppression
- [ ] Custom Whisper fine-tuning

---

**Quick Reference Card:**

```bash
# BUILD
cd rust_tui && cargo build --release

# RUN
./target/release/rust_tui

# TEST VOICE
Press 'v', speak, check status line

# CHECK LOGS
tail -f /tmp/codex_voice_tui.log

# FAST MODE
./target/release/rust_tui --seconds 2 --model ../models/ggml-tiny.en.bin
```