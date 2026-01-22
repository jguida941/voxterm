# Codex Voice Quick Start (Overlay Mode)

This is the shortest path to run Codex with voice in your terminal.

## 1) One-time setup

- Install Codex CLI:

```bash
npm install -g @anthropic-ai/codex
```

- Install Rust (if you don’t have it): https://rustup.rs

## 2) Run from any project

From your project folder:

```bash
/path/to/codex-voice/start.sh
```

That launcher will:
- Download a Whisper model if missing
- Start the Rust overlay
- Run Codex in your current folder

## 3) Voice controls

- `Ctrl+R` – start voice capture
- `Ctrl+V` – toggle auto-voice
- `Ctrl+Q` – exit overlay
- `Ctrl+C` – forwarded to Codex

## Common flags

```bash
/path/to/codex-voice/start.sh --auto-voice
/path/to/codex-voice/start.sh --voice-send-mode insert
/path/to/codex-voice/start.sh --prompt-regex '^codex> $'
```

## Homebrew (optional)

```bash
brew tap jguida941/homebrew-codex-voice
brew install codex-voice
$(brew --prefix)/opt/codex-voice/libexec/scripts/setup.sh models --base
cd ~/my-project
codex-voice
```

## Troubleshooting

- If voice falls back to Python and fails, install native Whisper model and run again:
  `./scripts/setup.sh models --base`
- Force native Whisper only:
  `./start.sh --no-python-fallback`
- Logs: `${TMPDIR}/codex_voice_tui.log`
- Prompt detection log: `${TMPDIR}/codex_overlay_prompt.log`
