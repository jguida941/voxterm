# How to Validate Audio Recording

This guide exercises the native Rust audio pipeline (`cpal` + resampler) and verifies that the voice capture stack is healthy before running full end-to-end tests.

## 1. Sanity Check: CLI Options
```bash
cd rust_tui
cargo run --release -- --help | grep seconds
```
Confirm the binary lists `--seconds`, `--input-device`, and `--whisper-model-path`. If the command fails, fix build issues before continuing.

## 2. Standalone Audio Capture Test
```bash
cd rust_tui
cargo run --bin test_audio
```
- Records ~3 seconds from the selected default microphone.
- Prints the device name, sample rate, channel count, and RMS levels.
- Success criteria: RMS > 0 and no panic.
- Failure troubleshooting:
  - Ensure macOS/Linux permissions grant microphone access to your terminal.
  - Specify a device explicitly: `INPUT_DEVICE="MacBook Pro Microphone" cargo run --bin test_audio`.

## 3. Full Voice Capture in the TUI
```bash
cd rust_tui
cargo run --release -- \
  --seconds 5 \
  --whisper-model-path ../models/ggml-base.en.bin \
  --log-timings
```
Steps:
1. Press `Ctrl+R` and speak for a few seconds.
2. Check that the transcript appears in the prompt buffer.
3. Press `Enter` to send it to Codex (or edit first).
4. Inspect `${TMPDIR}/codex_voice_tui.log` for entries similar to:
   ```
   timing|phase=voice_capture|record_s=5.000|stt_s=1.234|chars=57
   ```

## 4. Log Diagnostics
```bash
LOG_FILE="$(python - <<'PY'
import tempfile, pathlib
print(pathlib.Path(tempfile.gettempdir())/"codex_voice_tui.log")
PY)"

tail -50 "$LOG_FILE"
```
Look for:
- `capture_voice_native: Recorded XXXX samples ...`
- `capture_voice_native: Transcription complete ...`
- `Voice capture completed successfully`

If you see `native pipeline unavailable` errors, double-check the Whisper model path and confirm the `test_audio` binary captured sound.

## 5. Python Fallback (optional)
If native capture is temporarily unavailable, you can still validate the legacy pipeline:
```bash
python3 ../codex_voice.py --seconds 5 --emit-json --no-codex
```
Use this only for debugging; the production path should remain inside the Rust pipeline.

## 6. When to Update This Guide
Whenever audio code changes (new resampler, device selection tweaks, logging formats), rerun the commands above and update the text with the new expectations. Document the verification in the current day’s `docs/architecture/YYYY-MM-DD/` folder.
