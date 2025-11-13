# Python Legacy Reference

Historic commands for the original `codex_voice.py` pipeline. Keep for compatibility/debugging, but the Rust pipeline is now authoritative.

## Smoke Test (No Hardware)
```bash
python3 codex_voice.py \
  --seconds 1 \
  --ffmpeg-cmd ./stubs/fake_ffmpeg \
  --whisper-cmd ./stubs/whisper \
  --codex-cmd ./stubs/fake_codex \
  --auto-send \
  --emit-json
```

## Real Hardware Run
```bash
python3 codex_voice.py \
  --seconds 6 \
  --ffmpeg-cmd ffmpeg \
  --ffmpeg-device ":0" \
  --whisper-cmd whisper \
  --whisper-model base \
  --codex-cmd codex
```

## Flags Worth Remembering
- `--auto-send` – skip the edit prompt (hands-free)
- `--emit-json` – print transcript/prompt/codex_output/metrics to stdout
- `--no-codex` – stop after transcription (debug audio only)
- `--codex-arg=<flag>` – forward extra Codex CLI flags (repeatable)
- `--input-device "Device Name"` – override microphone selection

## When to Use
- Debugging audio issues before the Rust pipeline is configured
- Comparing Python vs. Rust behavior during regressions
- Reusing the JSON output in downstream scripts

Document any deviations between Python and Rust behavior in the daily architecture notes.
