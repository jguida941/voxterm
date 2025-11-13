# Troubleshooting Cheat Sheet

Common issues and the commands/logs we use to diagnose them. Update this file when a new problem/solution pair is discovered.

## Voice capture never finishes
- Make sure you press Ctrl+R (not 'v') inside the TUI.
- Confirm `${TMPDIR}/codex_voice_tui.log` shows `capture_voice_native`.
- Run `cargo run --bin test_audio` to ensure the microphone is actually recording.

## Python fallback triggered unexpectedly
- Check that `--whisper-model-path` points to an existing GGML file.
- Run `cargo run --release -- --list-input-devices` and pick the correct microphone with `--input-device`.
- Look for errors like `native pipeline unavailable` in the log file and note them in the daily architecture entry.

## Terminal stuck after running scripts
- Ensure helper scripts (e.g., `test_voice.sh`) terminate the TUI with Ctrl+C and do not leave background processes running.
- Use `pkill -f rust_tui` only as a last resort, and document it in the daily notes if you do.
