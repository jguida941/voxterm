# Codex Voice – Master Guide

This document is the high-level entry point for developers. For day-by-day design decisions and traceability, start with `PROJECT_OVERVIEW.md`, `master_index.md`, and the latest folder under `docs/architecture/YYYY-MM-DD/`.

## 1. Quick Start

```bash
# Clone once
git clone https://github.com/jguida941/codex_voice.git
cd codex_voice/rust_tui

# Build (debug)
cargo build

# Build + run (release)
cargo run --release -- --seconds 5 \
  --whisper-model-path ../models/ggml-base.en.bin
```

### Whisper Models
Download a GGML model (tiny/base/small/etc.) into `models/` and pass its path via `--whisper-model-path`. The TUI refuses to start the native pipeline without a model.

```bash
curl -L "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin" \
     -o models/ggml-base.en.bin
```

### Helpful CLI Flags
- `--seconds <n>` – recording duration (default 5s)
- `--input-device "Device Name"` – pick a specific microphone
- `--list-input-devices` – enumerate microphones and exit
- `--codex-arg="--danger-full-access"` – forward extra Codex CLI flags safely
- `--log-timings` – emit phase timing info to the log file
- `--no-python-fallback` – enforce Rust pipeline only (error if native fails)

## 2. Controls & UX
- `Ctrl+R` – start an immediate voice capture
- `Ctrl+V` – toggle automatic voice capture after every Codex reply
- `Enter` – send the current prompt to Codex
- `Esc` – clear the prompt
- `PageUp/PageDown` or `K/J` – scroll Codex output
- `Ctrl+C` – exit the TUI

After voice capture, the transcript appears in the prompt buffer so you can edit before pressing Enter.

## 3. Current Status (Nov 12, 2025)

### ✅ Working Today
- Native Rust audio → Whisper pipeline (`cpal` + `whisper-rs`)
- Persistent Codex PTY session (auto-started; falls back to one-shot CLI)
- Python pipeline remains as a safety net when native components are missing
- Structured logging written to `${TMPDIR}/codex_voice_tui.log`

### 🚧 In Progress
- Silence-aware capture + overlapped STT (latency reduction)
- Decomposing `app.rs`/`pty_session.rs` into <300 LOC modules
- Richer diagnostics (async logging, timing traces, CI checks)

### ⚠️ Known Gaps
- Default capture still records the full `--seconds` duration; no early-stop yet
- Python fallback must be retired once native pipeline is bulletproof
- Streaming Codex output is planned but currently snapshots after each response

Track progress through `docs/architecture/YYYY-MM-DD/` and the repo `CHANGELOG.md`.

## 4. Architecture Snapshot
For diagrams and detailed component breakdowns, see `docs/guides/architecture_overview.md`. Highlights:
- `audio.rs` handles device discovery, downmixing, and resampling (16 kHz mono)
- `stt.rs` loads whisper.cpp GGML models via `whisper-rs`
- `voice.rs` orchestrates capture + transcription on a worker thread
- `pty_session.rs` manages the Codex PTY, responds to CSI queries, and keeps the session alive
- `app.rs` coordinates UI state, PTY interactions, and voice triggers

## 5. Dependencies
- **Rust toolchain** (stable, currently 1.88.0)
- **GGML Whisper models** (place in `models/`)
- **Codex CLI** (must already be installed/configured)
- **Python 3 + ffmpeg + whisper/whisper.cpp** only if you rely on the fallback pipeline

## 6. Testing & Verification

### Manual Voice Test
```bash
cargo run --release -- \
  --seconds 6 \
  --whisper-model-path ../models/ggml-base.en.bin
```
Speak twice (e.g., introduce yourself, then ask Codex to recall it). Confirm Codex responses in the output pane.

### Audio Standalone Test
```bash
cargo run --bin test_audio
```
The helper binary captures ~3 seconds and prints RMS levels so you can verify that `cpal` is capturing audio.

### Unit Tests
```bash
cargo test --all
```

## 7. Persistence Check
Ensure the persistent Codex session is alive:
```bash
ps aux | grep codex | grep -v grep
```
You should see a single Codex process whose PID stays stable between prompts. Disable persistence with `--no-persistent-codex` if you need one-shot invocations for debugging.

## 8. Documentation Map
- `PROJECT_OVERVIEW.md` – mission + current focus
- `master_index.md` – navigation + daily checklist
- `docs/architecture/YYYY-MM-DD/` – daily design notes + changelog
- `docs/guides/*.md` – living references (this file, developer guide, audio testing guide, etc.)
- `docs/audits/*.md` – external/internal audits and responses

Always update the current day’s architecture folder and the repo changelog when you modify code or docs.
