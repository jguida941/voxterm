# VoxTerm Quick Start (Overlay Mode)

This is the shortest path to run Codex (default backend) with voice in your terminal.
Supported on macOS and Linux (use WSL2 if you are on Windows).

## 1) Prerequisites

- Install Codex CLI:

```bash
npm install -g @openai/codex
```

- Install Rust (if you do not have it): https://rustup.rs

## 2) Install VoxTerm

```bash
git clone https://github.com/jguida941/voxterm.git
cd voxterm
./install.sh
```

If `voxterm` is not found, see PATH and install notes in
[docs/INSTALL.md](docs/INSTALL.md).

macOS app alternative (folder picker): double-click **VoxTerm.app** and choose your project.

## 3) Run from any project

```bash
cd ~/my-project
voxterm
```

First run downloads a Whisper model if missing.

To target another AI CLI instead of Codex, pass `--backend`:

```bash
voxterm --backend claude
```

## 4) Essential controls

- `Ctrl+R` - start voice capture
- `Ctrl+V` - toggle auto-voice (disabling cancels any running capture)
- `Ctrl+T` - toggle send mode (auto vs insert)
- `Ctrl+Y` - open theme picker
- `Ctrl+O` - open settings menu (use ↑↓←→ + Enter)
- `Ctrl+]` - increase mic threshold by 5 dB (less sensitive)
- `Ctrl+\` - decrease mic threshold by 5 dB (more sensitive; `Ctrl+/` also works)
- `?` - show shortcut help
- `Ctrl+Q` - exit overlay
- `Ctrl+C` - forwarded to the CLI
- `Enter` - in insert mode, stop capture early and transcribe what was captured

Full behavior notes and screenshots are in [docs/USAGE.md](docs/USAGE.md).

## 5) Common flags

```bash
voxterm --auto-voice
voxterm --voice-send-mode insert
voxterm --voice-vad-threshold-db -50
voxterm --mic-meter
voxterm --logs
voxterm --voice-max-capture-ms 60000 --voice-buffer-ms 60000
voxterm --transcript-idle-ms 250
voxterm --prompt-regex '^codex> $'
```

See [docs/CLI_FLAGS.md](docs/CLI_FLAGS.md) for the full CLI flag and env var list.

## 6) Need help?

- Install options: [docs/INSTALL.md](docs/INSTALL.md)
- Troubleshooting: [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)
