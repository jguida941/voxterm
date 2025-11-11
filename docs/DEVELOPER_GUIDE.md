# Developer Guide

Central place for the exact commands and settings we use while developing/test‑driving
Codex Voice. Everything below assumes you are in the repo root.

---

## 1. Python Pipeline (codex_voice.py)

These commands exercise the canonical voice → Whisper → Codex pipeline. Run them before
touching the Rust TUI so you know the backend is healthy.

### 1.1 Stubbed smoke test (no hardware/Codex needed)

```bash
python codex_voice.py \
  --seconds 1 \
  --ffmpeg-cmd ./stubs/fake_ffmpeg \
  --whisper-cmd ./stubs/whisper \
  --codex-cmd ./stubs/fake_codex \
  --auto-send \
  --emit-json
```

Expected: a single JSON blob plus the stub’s `Codex received prompt` line.

### 1.2 Real microphone + Whisper + Codex

```bash
python codex_voice.py \
  --seconds 6 \
  --ffmpeg-cmd ffmpeg \
  --ffmpeg-device ":0" \        # adjust for Linux/Windows devices
  --whisper-cmd whisper \
  --whisper-model base \
  --codex-cmd codex
```

Speak when prompted, edit if needed, press Enter. The script prints `[Transcript]`,
`[Codex output]`, and `[Latency]` sections so you can gauge the end-to-end flow.

> **Whisper model files**
> Download a `ggml-*.bin` (e.g. `ggml-base.en.bin`) into `./models/` and point the TUI at
> it with `--whisper-model-path ./models/ggml-base.en.bin` or set the
> `WHISPER_MODEL_PATH` environment variable. The new Rust pipeline will refuse to start if
> it cannot load a model.

Need to override the microphone? Pass `--input-device "<device name>"` (names are shown in
`system_profiler SPAudioDataType`). The launcher also respects
`INPUT_DEVICE_OVERRIDE="MacBook Pro Microphone" ./voice`.

Flags to remember:

- `--auto-send` skips the edit prompt (hands-free mode).
- `--emit-json` prints `transcript/prompt/codex_output/metrics` as JSON for tooling.
- `--no-codex` stops after transcription to debug audio without touching Codex.
- `--codex-args "<flags>"` forwards extra Codex CLI options (e.g. `--danger-full-access`).
- `--codex-arg=<flag>` repeatable form so you don’t have to worry about shell quoting; keep the `=` when the value itself starts with `--`.
- `--log-timings` (TUI only) emits detailed timing metrics into `${TMPDIR}/codex_voice_tui.log`.
- `--no-persistent-codex` disables the long-lived Codex PTY session if it misbehaves.

---

## 2. Rust TUI

`./voice` (or `./scripts/run_tui.sh`) launches the ratatui interface. Voice capture is now
handled entirely in Rust (cpal audio + whisper-rs), so the TUI no longer shells out to
`codex_voice.py`. You must provide a Whisper model path (e.g.
`--whisper-model-path ./models/ggml-base.en.bin`) so the native pipeline can load it.
The launcher still accepts extra Codex CLI flags through `CODEX_ARGS_OVERRIDE`, so you can
enable dangerous permissions without editing sources.

### 2.1 Default launch

```bash
./voice
```

### 2.2 Launch with extra Codex flags (e.g. full file access)

```bash
CODEX_ARGS_OVERRIDE="--danger-full-access" ./voice
# or without the wrapper
CODEX_ARGS_OVERRIDE="--danger-full-access" ./scripts/run_tui.sh
```

The launcher converts `CODEX_ARGS_OVERRIDE` into repeated `--codex-arg` parameters for the
Rust binary, and the TUI threads those through every Codex invocation (PTY helper,
interactive fallback, `codex exec`).

### 2.3 Log inspection

```bash
tail -f "${TMPDIR:-/tmp}/codex_voice_tui.log"
```

Useful while reproducing Enter/voice issues. The TUI logs key events, voice capture
status, and Codex invocation attempts.

### 2.4 Timing & persistence toggles

- `LOG_TIMINGS_OVERRIDE=1 ./voice` enables high-detail timing logs (voice and Codex phases).
- `DISABLE_PERSISTENT_CODEX_OVERRIDE=1 ./voice` falls back to one-shot Codex invocations.

---

## 3. Typical Debug Loop

1. Run the stubbed Python command (Section 1.1). If it fails, fix the pipeline before
   touching the TUI.
2. Run the real Python command (Section 1.2) to confirm your microphone/Whisper/Codex
   stack.
3. Launch `./voice` (Section 2) and exercise the same sequence inside the TUI.
4. Watch the log (`tail -f …codex_voice_tui.log`) if input or Codex calls misbehave.

This order prevents chasing TUI UI bugs when the root cause is an audio or Codex issue.

---

## 4. Handy Environment Overrides

| Variable                 | Purpose                                             |
|-------------------------|-----------------------------------------------------|
| `SECONDS_OVERRIDE`      | Recording duration passed to the TUI launcher       |
| `FFMPEG_DEVICE_OVERRIDE`| Override microphone device string for ffmpeg        |
| `WHISPER_MODEL_OVERRIDE`| Change Whisper model (tiny/base/small/…)            |
| `CODEX_CMD_OVERRIDE`    | Alternate Codex binary                              |
| `CODEX_ARGS_OVERRIDE`   | Extra Codex CLI flags (space-separated)             |
| `TERM_OVERRIDE`         | TERM value exported before running Codex            |
| `PYTHON_CMD_OVERRIDE`   | Python interpreter for PTY helper (Codex PTY only)  |
| `WHISPER_MODEL_PATH`    | Path to ggml*.bin for the native Whisper pipeline   |
| `INPUT_DEVICE_OVERRIDE` | Name of audio input device (see `system_profiler SPAudioDataType`) |
| `PIPELINE_SCRIPT_OVERRIDE` | Deprecated: legacy python pipeline path (unused) |
| `LOG_TIMINGS_OVERRIDE`  | Set (e.g. `1`) to pass `--log-timings` to the TUI   |
| `DISABLE_PERSISTENT_CODEX_OVERRIDE` | Set to disable the persistent Codex PTY session |

All env overrides feed into `scripts/run_tui.sh` and therefore the `./voice` entry point.
`PIPELINE_SCRIPT_OVERRIDE` saves you from manually passing `--pipeline-script` when testing
an alternate backend (the default remains `../codex_voice.py`).

---

Keep this guide updated whenever we add new flags, scripts, or debugging tricks so every
developer can reproduce the exact workflow used during development.
