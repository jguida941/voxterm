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
┌─────────────────────────────┐   ┌──────────────────────────┐
│      RUST TUI (Main)        │   │ Voice Input (Ctrl+R)      │
│   rust_tui/src/main.rs      │   │  • Captures audio         │
│   - Terminal UI (ratatui)   │◄──┤  • Transcribes w/ Whisper │
│   - Event loop              │   │  • Sends prompt to Codex  │
│   - Status display          │   └──────────────────────────┘
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
├── codex_voice.py             # Python fallback pipeline (legacy but available)
├── scripts/                    # Helper scripts (launchers, PTY utilities)
└── docs/guides/architecture_overview.md  # This guide

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
# Show all options (from rust_tui/)
cargo run --release -- --help

# Common flags
--seconds 5                          # Recording duration (default 5s)
--lang en                            # Whisper language/tokens
--input-device "MacBook Pro Microphone"   # Force a specific microphone
--list-input-devices                 # Enumerate microphones and exit
--log-timings                        # Emit timing breakdown to log file
--whisper-model-path ../models/ggml-base.en.bin   # GGML file for whisper-rs
--codex-cmd /path/to/codex           # Codex CLI binary
--codex-arg="--danger-full-access"   # Forward extra Codex CLI flags safely
--no-python-fallback                 # Disable Python pipeline (error if native fails)
```

### Examples
```bash
# Fast setup (2 s recording, tiny model)
cargo run --release -- \
  --seconds 2 \
  --whisper-model-path ../models/ggml-tiny.en.bin

# High quality (5 s recording, base model)
cargo run --release -- \
  --seconds 5 \
  --whisper-model-path ../models/ggml-base.en.bin

# Debug performance issues with timings enabled
cargo run --release -- \
  --log-timings \
  --whisper-model-path ../models/ggml-base.en.bin

# Use a specific microphone and custom Codex flag
cargo run --release -- \
  --input-device "MacBook Pro Microphone" \
  --codex-arg="--danger-full-access" \
  --whisper-model-path ../models/ggml-base.en.bin
```

## 🎤 Voice Pipeline Decision Tree

```
User presses Ctrl+R (or auto voice mode triggers)
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
- Uses `cpal` for cross-platform audio capture
- Downmixes to mono and recenters unsigned samples
- Resamples to 16 kHz (Whisper requirement)
- Two resamplers:
  - High-quality: Rubato (feature-gated)
  - Basic: FIR + linear interpolation

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

#### Key Bindings
- `Ctrl+R` – start a voice capture immediately
- `Ctrl+V` – toggle automatic voice capture after each Codex reply
- `Enter` – send the current prompt to Codex
- `Esc` – clear the input buffer
- `PageUp/PageDown` or `K/J` – scroll Codex output
- `Ctrl+C` – exit the TUI

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
cargo run --release -- --log-timings --whisper-model-path ../models/ggml-base.en.bin

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
--whisper-model-path ../models/ggml-tiny.en.bin

# Reduce recording time
--seconds 2

# Ensure release build
cargo build --release

# Prefer native Rust path (avoid Python fallback)
--no-python-fallback

# Select the best audio device
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
cargo test
cargo test --features high-quality-audio
cargo test voice::tests::python_fallback_returns_trimmed_transcript
```

### Manual Integration Test
```bash
# Enumerate microphones
cargo run --release -- --list-input-devices

# Full TUI + voice capture
cargo run --release -- \
  --seconds 5 \
  --whisper-model-path ../models/ggml-base.en.bin
```
1. Press `Ctrl+R`, speak, and confirm the transcript appears in the prompt.
2. Press `Enter` to send it to Codex and verify the response in the output pane.
3. Review `${TMPDIR}/codex_voice_tui.log` for `timing|phase=voice_capture` entries.

## 📊 Status Line Indicators

- **Ready. Press Ctrl+R...** – waiting for input
- **Recording voice...** – `audio.rs` actively capturing samples
- **Transcribing...** – Whisper inference running
- **Rust pipeline/Python fallback** – which STT path produced the transcript
- **Errors** – surfaced directly (e.g., microphone permissions)

## 🎯 Common Issues & Solutions

### "Fallback to Python pipeline" when native should work
- Ensure `--whisper-model-path` points to an existing GGML file.
- Check `cargo run --bin test_audio` to confirm the microphone captures anything.

### No devices listed / permission errors
- On macOS grant microphone permission to your terminal (System Settings → Privacy & Security → Microphone).
- Provide a device explicitly with `--input-device` to avoid default device confusion.

### Voice capture feels slow
- Reduce `--seconds` (recording duration) until silence-aware capture lands.
- Enable `--log-timings` and attach the log when filing performance bugs.

### Issue: Panic with large byte index
**Solution:** Already fixed! Our patches handle:
- UTF-8 safe string slicing
- Saturating arithmetic
- Mutex poisoning recovery
- Terminal query responses

### Issue: Slow transcription
**Solution:** Use the tiny model + shorter recording
```bash
cargo run --release -- \
  --seconds 2 \
  --whisper-model-path ../models/ggml-tiny.en.bin
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
cargo run --release -- --whisper-model-path ../models/ggml-base.en.bin

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
cargo run --release -- --whisper-model-path ../models/ggml-base.en.bin

# TEST VOICE
Press Ctrl+R, speak, verify transcript, press Enter

# CHECK LOGS
tail -f /tmp/codex_voice_tui.log

# FAST MODE (short capture + tiny model)
cargo run --release -- \
  --seconds 2 \
  --whisper-model-path ../models/ggml-tiny.en.bin
```
