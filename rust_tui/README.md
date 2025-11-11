# Codex Voice TUI

Rust terminal UI that wraps the Codex Voice pipeline. It mirrors the Python prototype but uses `ratatui` + `crossterm` for rendering, `cpal` for cross‑platform audio capture, and `whisper-rs` for local STT. The app keeps a persistent Codex PTY session so tool calls and shell state survive between prompts, while falling back to the original Python pipeline any time native capture or transcription fails.

## Features

- Persistent PTY-backed Codex session with full terminal emulation
- Optional high-quality resampling powered by `rubato`
- Local Whisper transcription with transparent Python fallback
- Scrollback-limited TUI with keyboard shortcuts for sending prompts, paging, and toggling voice capture
- Structured logging (`log_timings`, `log_debug`) for performance investigations

## Quick Start

```bash
cd rust_tui
cargo run -- --seconds 5 --lang en --codex-cmd codex
```

Helpful flags (run `cargo run -- --help` for the full list):

| Flag | Purpose |
| ---- | ------- |
| `--input-device` | Force a specific microphone instead of the OS default |
| `--list-input-devices` | Print detected microphones and exit without launching the TUI |
| `--no-persistent-codex` | Disable the PTY session and spawn Codex per prompt |
| `--codex-arg <ARG>` | Forward extra flags to the Codex CLI (validated/size-limited) |
| `--whisper-model-path <PATH>` | Point at a local ggml model (auto-detected from `../models/` otherwise) |
| `--ffmpeg-device <NAME>` | Override the device passed to the Python recorder |
| `--log-timings` | Emit timing summaries to `~/.cache/codex_voice_tui.log` |

### Voice Capture Flow

1. Recorder (`audio::Recorder`) captures raw PCM via `cpal` and down-mixes/ resamples to 16 kHz mono.
2. `stt::Transcriber` loads Whisper (once) and transcribes the samples on a background thread.
3. Failures or missing components fall back to `codex_voice.py`, which streams JSON back into the TUI.
4. Prompts go through the PTY session (`pty_session::PtyCodexSession`) when `--persistent-codex` is enabled; otherwise we spawn `codex exec -`.

### Keyboard Shortcuts (default bindings)

- `Enter` – send typed prompt
- `Ctrl+R` – start voice capture immediately
- `Ctrl+V` – toggle automatic capture after each Codex response
- `PageUp/PageDown` or `K/J` – scroll the Codex output window
- `Esc` – clear input

## Project Layout

| Module | Responsibility |
| ------ | -------------- |
| `src/app.rs` | Core App state + event handlers shared by UI + worker threads |
| `src/ui.rs` | `ratatui` widgets and layout |
| `src/audio.rs` | Device discovery, capture, and (optional) high-quality resampling |
| `src/stt.rs` | Whisper wrapper with stderr suppression |
| `src/voice.rs` | Voice job orchestration and fallback management |
| `src/pty_session.rs` | Unsafe PTY glue that manages the Codex child process |

Run `cargo doc --open` for rustdoc on all modules; most public APIs have usage notes inline.

## Development Workflow

```bash
# Format + lint
cargo fmt
cargo clippy --workspace --all-features

# Run the full suite (unit tests live inside modules)
cargo test
```

Some tests mock OS interactions (e.g., PTY escape handling) but anything that touches real audio hardware is exercised through the CLI rather than unit tests.

## Troubleshooting

- **`no samples captured`** – ensure microphone permissions are granted, or pass `--input-device` to pick the right device.
- **Whisper model not found** – either drop a ggml model into `../models/` or pass both `--whisper-model` and `--whisper-model-path`.
- **Codex command fails** – verify `--codex-cmd` is executable and that any repeated `--codex-arg` values are listed in the allowlist; the CLI validates paths and byte limits up front.
- **PTY session issues** – use `--no-persistent-codex` to fall back to one-off processes while debugging, or inspect `$(mktemp)/codex_voice_tui.log` for the PTY logs.

## CI Expectations

The repository uses `cargo fmt`, `cargo clippy --all-features`, and `cargo test` as the baseline gates. See `.github/workflows/rust_tui.yml` for the exact command matrix.
