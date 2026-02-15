# Troubleshooting

Use this page as a navigation hub. Pick your issue category first, then jump to
the focused guide for faster diagnosis.

## Quick Fixes

| Problem | First action | Detailed guide |
|---------|--------------|----------------|
| No speech detected | Lower mic threshold (`Ctrl+\\`) | [Status Messages](#status-messages) |
| Voice not recording | Check microphone permissions | [Audio Setup](#audio-setup) |
| Codex/Claude not responding | Verify install + login | [Backend Issues](TROUBLESHOOTING_BACKEND.md#codex-not-responding) |
| Auto-voice not triggering | Check prompt detection | [Backend Issues](TROUBLESHOOTING_BACKEND.md#auto-voice-not-triggering) |
| Transcript queued while backend is busy | Wait for prompt or tune prompt regex | [Backend Issues](TROUBLESHOOTING_BACKEND.md#transcript-queued-n) |
| Wrong version after update | Check PATH + reinstall flow | [Install Issues](TROUBLESHOOTING_INSTALL.md#wrong-version-after-update) |
| HUD duplicates/flickers in JetBrains | Verify version and collect logs | [Terminal/IDE Issues](TROUBLESHOOTING_TERMINAL.md#hud-duplicates-in-jetbrains-terminals) |
| Startup splash behaves oddly in IDE terminal | Tune splash env vars | [Terminal/IDE Issues](TROUBLESHOOTING_TERMINAL.md#startup-banner-lingers-in-ide-terminal) |
| Theme colors look muted | Verify truecolor env | [Terminal/IDE Issues](TROUBLESHOOTING_TERMINAL.md#theme-colors-look-muted-in-ide-terminal) |
| `PTY write failed: Input/output error` on exit | Usually benign race during shutdown | [Terminal/IDE Issues](TROUBLESHOOTING_TERMINAL.md#pty-exit-write-error-in-logs) |

## Contents

- [Pick by Category](#pick-by-category)
- [Status Messages](#status-messages)
- [Audio Setup](#audio-setup)
- [Mic Sensitivity](#mic-sensitivity)
- [Enabling Logs](#enabling-logs)
- [FAQ](#faq)
- [Getting Help](#getting-help)
- [See Also](#see-also)

## Pick by Category

- Backend behavior (`codex`/`claude`, prompt detection, queueing):
  [TROUBLESHOOTING_BACKEND.md](TROUBLESHOOTING_BACKEND.md)
- Terminal and IDE rendering/input behavior:
  [TROUBLESHOOTING_TERMINAL.md](TROUBLESHOOTING_TERMINAL.md)
- Install and upgrade issues:
  [TROUBLESHOOTING_INSTALL.md](TROUBLESHOOTING_INSTALL.md)

---

## Status Messages

### No speech detected

The mic recorded but no voice crossed the current threshold.

1. Speak louder or closer to the mic.
2. Lower threshold with `Ctrl+\\` (or `Ctrl+/`).
3. Run `voiceterm --mic-meter` to calibrate.

### Voice capture failed (see log)

Capture could not start.

1. Check mic permissions for your terminal app.
2. Run `voiceterm --list-input-devices`.
3. Try a specific device: `voiceterm --input-device "Your Mic Name"`.
4. Re-run with logs: `voiceterm --logs`.

### Voice capture error (see log)

Recording or transcription failed at runtime.

1. Run with logs: `voiceterm --logs`.
2. Check `${TMPDIR}/voiceterm_tui.log` (macOS) or `/tmp/voiceterm_tui.log` (Linux).
3. Restart `voiceterm`.

### Processing... (stuck)

Transcription is taking longer than expected.

1. Wait up to 60 seconds for longer captures.
2. If still stuck, press `Ctrl+C` and restart.
3. Try a smaller model (`--whisper-model base`).

### Voice macro not expanding

Macros load from `<project>/.voiceterm/macros.yaml` and apply only when
`Settings -> Macros` is ON.

1. Confirm file path and YAML structure.
2. Check trigger text match (case/whitespace-insensitive).
3. Restart VoiceTerm after editing macros.

### Voice macro expanded unexpectedly

Macro expansion runs only when `Settings -> Macros` is ON.

1. Open Settings (`Ctrl+O`).
2. Set `Macros` to OFF for raw transcript injection.

### Python pipeline

Native STT was unavailable, so Python fallback was used.

1. Check models: `ls whisper_models/ggml-*.bin`
2. Download model: `./scripts/setup.sh models --base`
3. Ensure fallback dependencies exist (`python3`, `ffmpeg`, `whisper`) or use
   `--no-python-fallback`.

---

## Audio Setup

### Check microphone permissions

- macOS: System Settings -> Privacy & Security -> Microphone
- Linux: verify PulseAudio/PipeWire access (`pactl list sources`)

### Verify Whisper model exists

```bash
ls whisper_models/ggml-*.bin
```

If missing:

```bash
./scripts/setup.sh models --base
```

### List/select devices

```bash
voiceterm --list-input-devices
voiceterm --input-device "MacBook Pro Microphone"
```

### Microphone changed or unplugged

Restart VoiceTerm after plugging in a new input device.

---

## Mic Sensitivity

### Too sensitive (background noise triggers capture)

- Press `Ctrl+]` to raise threshold (less sensitive)
- Or set startup threshold:

```bash
voiceterm --voice-vad-threshold-db -30
```

### Not sensitive enough (misses your voice)

- Press `Ctrl+\\` (or `Ctrl+/`) to lower threshold (more sensitive)
- Or set startup threshold:

```bash
voiceterm --voice-vad-threshold-db -50
```

### Find the right threshold

```bash
voiceterm --mic-meter
```

Hotkey range: `-80 dB` to `-10 dB`, default `-55 dB`.
CLI flag range: `-120 dB` to `0 dB`.

---

## Enabling Logs

```bash
voiceterm --logs
```

Include transcript snippets (optional):

```bash
voiceterm --logs --log-content
```

Disable all logs:

```bash
voiceterm --no-logs
```

Log paths:
- Debug log: `${TMPDIR}/voiceterm_tui.log` (macOS) or `/tmp/voiceterm_tui.log` (Linux)
- Trace log: `${TMPDIR}/voiceterm_trace.jsonl` or `/tmp/voiceterm_trace.jsonl`

---

## FAQ

### What languages does Whisper support?

Whisper supports many languages. Start with `--lang en` (tested) or use
`--lang auto`.

Reference:
[Whisper supported languages](https://github.com/openai/whisper#available-models-and-languages)

### Which AI CLI backends work?

Canonical backend support matrix lives in:
[USAGE.md -> Backend Support](USAGE.md#backend-support)

### Which Whisper model should I use?

Start with `base` for speed, `small` for balance, `medium` for higher accuracy.
See [WHISPER.md](WHISPER.md) for full guidance.

### Can I use VoiceTerm without Codex?

Yes. Use Claude:

```bash
voiceterm --claude
```

### Does VoiceTerm send voice audio to the cloud?

No. STT runs locally via Whisper.

### How do I update VoiceTerm?

Homebrew:

```bash
brew update && brew upgrade voiceterm
```

Source install:

```bash
cd voiceterm && git pull && ./scripts/install.sh
```

---

## Getting Help

- Collect diagnostics: `voiceterm --doctor`
- Report issues: [GitHub Issues](https://github.com/jguida941/voiceterm/issues)
- Check active known issues: [Master Plan](../dev/active/MASTER_PLAN.md)

---

## See Also

| Topic | Link |
|-------|------|
| Backend issues | [TROUBLESHOOTING_BACKEND.md](TROUBLESHOOTING_BACKEND.md) |
| Terminal/IDE issues | [TROUBLESHOOTING_TERMINAL.md](TROUBLESHOOTING_TERMINAL.md) |
| Install/update issues | [TROUBLESHOOTING_INSTALL.md](TROUBLESHOOTING_INSTALL.md) |
| Install guide | [INSTALL.md](INSTALL.md) |
| Usage guide | [USAGE.md](USAGE.md) |
| CLI flags | [CLI_FLAGS.md](CLI_FLAGS.md) |
| Whisper guide | [WHISPER.md](WHISPER.md) |
