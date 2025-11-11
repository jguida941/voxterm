# Codex Voice - Master Documentation

> **Docs index:** Supporting guides now live under `docs/` (e.g., `docs/README.md`, `docs/DEVELOPER_GUIDE.md`, `docs/HOW_TO_TEST_AUDIO.md`, `docs/chatgpt.md`). Use this master doc as the entry point and follow the links out to the new locations.

## Quick Start

### Install & Run
```bash
# Clone repo
git clone https://github.com/jguida941/codex-voice.git
cd codex-voice

# Run with voice (10 second recording)
./voice

# Run with custom duration
./voice -s 15  # 15 seconds
./voice -s 30  # 30 seconds
```

### Controls
- `Ctrl+R` - Start voice recording
- `Enter` - Send message to Codex
- `Ctrl+C` - Exit

### Test Persistence
1. Say "Hello, my name is Bob"
2. Say "What is my name?"
3. If Codex remembers "Bob" = Working!

---

## Current Status

### ✅ What Works
- Voice capture and transcription (native Rust pipeline)
- Full Codex tool access (can edit files!)
- Configurable recording duration
- Multiple voice captures
- Enter key fixed
- Proper output formatting with newlines
- Full permission mode enabled (`--danger-full-access`)
- Persistent Codex PTY session (auto-started, falls back to single-shot CLI if it fails)

### 🚧 In Progress
- Streaming output over the PTY channel
- Timing instrumentation rollout (LOG_TIMINGS override)
- Native Whisper polish (model path setup, latency tuning)

### ❌ Known Issues
- 3-4 second delay (spawning new processes)
- Whisper process restarts per capture (no warm cache yet)

---

## Architecture

### The Problem
The old Python pipeline spawned 3 processes (FFmpeg → Whisper → Codex) and reloaded everything.
We now capture/transcribe in Rust, but still need to eliminate remaining latency (model load,
Codex PTY handshake) and keep everything alive.

### The Solution
- Native Rust audio/Whisper pipeline (done)
- Keep Codex session alive via PTY (in progress)
- Reuse Whisper state + introduce streaming output to hit <200ms

### Implementation Status
```
✅ Phase 1: Quick Fixes
   - Fixed Enter key bug
   - Fixed file editing access
   - Configurable duration

🚧 Phase 2: Persistent Sessions (IN PROGRESS)
   - PTY handler implemented
   - Session manager enabled by default (auto-fallback to one-shot CLI)
   - Requires more testing plus streaming output hooked back up

📋 Phase 3: Performance (TODO)
   - Replace FFmpeg with cpal
   - Integrate whisper.cpp
   - Zero-copy audio pipeline

📋 Phase 4: Features (TODO)
   - Wake word ("Hey Codex")
   - Voice activity detection
   - Streaming transcription
```

---

## Technical Details

### Dependencies
- **FFmpeg** - Audio recording
- **Whisper** - Speech-to-text (OpenAI)
- **Codex CLI** - The actual AI assistant
- **Rust** - TUI application

### Installation
```bash
# macOS
brew install ffmpeg
pip install openai-whisper

# Rust (for building)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cd rust_tui
cargo build --release
```

### Project Structure
```
codex-voice/
├── voice                  # Main executable
├── rust_tui/
│   ├── src/
│   │   ├── main.rs       # TUI application
│   │   ├── codex_session.rs  # Session management (old)
│   │   └── pty_session.rs    # PTY-based sessions (new)
│   └── Cargo.toml
├── scripts/
│   ├── run_tui.sh        # Launch script
│   └── run_in_pty.py     # PTY helper
└── docs/archive/         # Old documentation
```

---

## Testing

### Basic Voice Test
```bash
./voice -s 10

# Test 1: Context
Say: "My name is Alice"
Say: "What is my name?"
Expected: "Alice"

# Test 2: File Operations
Say: "Create a file called test.py with hello world"
Expected: File created

# Test 3: Multi-turn
Say: "What files are in this directory?"
Say: "Open the README file"
Expected: Shows README content
```

### Check Persistence
```bash
# While running, in another terminal:
ps aux | grep codex | grep -v grep

# Should show ONE codex process
# Same PID for all messages = persistent
# Different PIDs = not persistent (current issue)
```

For the exact step-by-step commands we use during development (stubbed Python smoke
tests, real mic runs, TUI launch overrides, log inspection), see
[`DEVELOPER_GUIDE.md`](docs/DEVELOPER_GUIDE.md).

---

## Common Issues & Fixes

### "Codex not found"
```bash
# Check if installed
which codex

# If not, install Codex CLI
# (Follow Anthropic's instructions)
```

### "Whisper not found"
```bash
# Activate venv
source .venv/bin/activate

# Or install
pip install openai-whisper
```

### "Can't edit files"
This is fixed! We removed `--skip-git-repo-check` flag.

### "Enter key doesn't work"
This is fixed! Event queue clearing implemented.

### "Too slow"
Still an issue. Persistent sessions will fix this (in progress).

---

## Roadmap

### Immediate (This Week)
- [x] Add timing instrumentation (LOG_TIMINGS flag + JSON metrics logging)
- [ ] Verify PTY session persistence works end-to-end
- [ ] Fix any session bugs found in testing
- [ ] Merge improvements to main

### Short Term (Next Week)
- [ ] Replace FFmpeg with Rust audio (cpal)
- [ ] Integrate whisper.cpp as library
- [ ] Implement streaming output

### Medium Term (Month)
- [ ] Wake word detection
- [ ] Voice activity detection
- [ ] Multi-language support
- [ ] Package as single binary

### Long Term
- [ ] Homebrew formula
- [ ] Cross-platform (Windows/Linux)
- [ ] GPU acceleration
- [ ] Cloud whisper option

---

## For Developers

### Build & Run
```bash
# Debug build
cd rust_tui
cargo build

# Release build (optimized)
cargo build --release

# Run with logging
RUST_LOG=debug cargo run
```

### Architecture Decisions
1. **Why Rust?** Speed, single binary, memory safety
2. **Why PTY?** Codex needs real terminal, not pipes
3. **Why not persistent yet?** PTY implementation just added, needs testing
4. **Why ratatui?** Best Rust TUI framework

### Contributing
1. Test the PTY implementation
2. Report bugs with logs
3. PRs welcome for performance improvements

---

## Summary

**What it is:** Voice interface for Codex CLI
**What works:** Voice → Text → Codex → Response
**What's broken:** No session persistence (yet)
**What's next:** Test PTY implementation for persistence

**Bottom line:** It works but needs the persistent session feature tested and debugged to be truly useful.

---

## Quick Commands Reference

```bash
# Run with voice
./voice

# Custom duration
./voice -s 20

# Test typing only
./test_typing.sh

# Test voice session
./test_voice_session.sh

# Check logs
cat $TMPDIR/codex_voice_tui.log

# Build
cd rust_tui && cargo build --release

# Clean build
cargo clean && cargo build
```

---

*Last Updated: November 6, 2024*
*Status: PTY implementation complete, needs testing*
