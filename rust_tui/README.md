# VoxTerm Backend

Rust backend for VoxTerm - handles audio capture, speech-to-text, and provider communication.

## Role

This is the **backend** component that:
- Captures audio via CPAL
- Transcribes speech using Whisper
- Communicates with Codex/Claude CLIs
- Exposes a JSON-IPC interface for external UI integrations (advanced)

## Building

```bash
cd rust_tui
cargo build --release
```

The main binary is at `target/release/rust_tui`.

Overlay binary (Codex PTY passthrough):

```bash
cd rust_tui
cargo build --release --bin voxterm
```

## Usage

### JSON IPC Mode (advanced)

```bash
./target/release/rust_tui --json-ipc
```

Communicates via JSON-lines on stdin/stdout.

### Standalone TUI Mode

Can also run as a standalone terminal UI:

```bash
cargo run --release -- --seconds 5 --whisper-model-path ../models/ggml-base.en.bin
```

### Overlay Mode

Runs Codex in a PTY and overlays voice status in your terminal:

```bash
./target/release/voxterm
```

## CLI Options

### Core Options
| Flag | Purpose |
|------|---------|
| `--json-ipc` | Run in JSON IPC mode (advanced) |
| `--codex-cmd <CMD>` | Path to Codex CLI [default: codex] |
| `--persistent-codex` | Enable PTY session for Codex |
| `--log-timings` | Enable verbose timing logs |

### Audio Options
| Flag | Purpose |
|------|---------|
| `--input-device <NAME>` | Preferred audio input device |
| `--list-input-devices` | Print available devices and exit |
| `--voice-vad-engine <ENGINE>` | VAD implementation: `earshot` or `simple` |
| `--voice-vad-threshold-db <DB>` | VAD threshold in decibels [default: -40] |
| `--seconds <N>` | Recording duration in seconds [default: 5] |

### Whisper Options
| Flag | Purpose |
|------|---------|
| `--whisper-model-path <PATH>` | Path to Whisper GGML model (required) |
| `--whisper-model <NAME>` | Whisper model name [default: small] |
| `--lang <LANG>` | Language for Whisper [default: en] |
| `--no-python-fallback` | Fail instead of using Python STT fallback |

Run `cargo run -- --help` for the full list of 30+ options.

## Architecture

```
src/
├── main.rs          # Entry point, CLI parsing
├── lib.rs           # Module exports
├── ipc.rs           # JSON-IPC protocol handler
├── codex.rs         # Codex/Claude backend (PTY + CLI modes)
├── voice.rs         # Voice capture orchestration
├── audio.rs         # CPAL recording, VAD
├── stt.rs           # Whisper transcription
├── pty_session/     # Unix PTY wrapper
├── config.rs        # Configuration and validation
├── utf8_safe.rs     # UTF-8 safety utilities
├── vad_earshot.rs   # Voice activity detection
├── ui.rs            # TUI rendering (standalone mode)
├── app.rs           # TUI state machine (standalone mode)
└── bin/             # Additional binaries
```

## Providers

### Claude

Uses `claude --print <prompt>` for simple, non-interactive responses.

### Codex

Two modes:
- **CLI mode** (default): `codex exec - -C <dir>` per prompt
- **PTY mode** (`--persistent-codex`): Persistent pseudo-terminal session

## Development

```bash
# Format
cargo fmt

# Lint
cargo clippy --all-features

# Test
cargo test

# Generate docs
cargo doc --open
```

## Troubleshooting

- **No audio captured**: Check microphone permissions, try `--list-input-devices`
- **Whisper model not found**: Ensure model exists at specified path
- **Provider not responding**: Verify CLI is installed and authenticated
- **PTY issues**: Try without `--persistent-codex`

Logs are written to `$TMPDIR/voxterm_tui.log`.
