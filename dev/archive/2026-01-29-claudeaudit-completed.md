# Completed Items - 2026-01-29 (Claude Audit Baseline)

Source: `claudeaudit.md` (2026-01-29). These items were already completed at the time the master plan was created.

## Memory and buffering
- [x] Bound overlay input/writer channels or add backpressure/drop policy. (`rust_tui/src/bin/codex_overlay.rs`)
- [x] Add max length cap for TUI input buffer. (`rust_tui/src/app.rs`)
- [x] Cap `combined_raw` growth in PTY call. (`rust_tui/src/codex.rs`)
- [x] Make voice job channel bounded or otherwise cap growth. (`rust_tui/src/voice.rs`)
- [x] Surface VAD frame drops to the user when non-zero. (`rust_tui/src/audio/recorder.rs`)

## Logging, disk, privacy
- [x] Default all logging OFF in production; enable only via explicit flags/env (dev mode).
- [x] Make prompt log opt-in (flag/env) and disable by default. (`rust_tui/src/bin/codex_overlay.rs`)
- [x] Add runtime size cap/rotation for prompt log. (`rust_tui/src/bin/codex_overlay.rs`)
- [x] Add runtime size cap/rotation for debug log, and avoid heavy per-write overhead. (`rust_tui/src/app.rs`)
- [x] Remove or gate partial prompt/response logging in IPC (`prompt[..30]`, `line[..50]`). (`rust_tui/src/ipc.rs`)
- [x] Add a `--no-logs` or equivalent privacy flag to disable content logging. (docs + code)

## Behavior and UX
- [x] Warn when transcript queue drops the oldest item. (`rust_tui/src/bin/codex_overlay.rs`)
- [x] Fix insert-mode queue stall (only advance `last_enter_at` when newline sent). (`rust_tui/src/bin/codex_overlay.rs`)
- [x] Keep auto-voice status visible while auto mode is enabled and actively listening. (status line)
- [x] Add platform-specific mic permission guidance in error messages. (`rust_tui/src/audio/recorder.rs`, docs)
- [x] Add device hotplug/recovery or document lack of hotplug support. (`rust_tui/src/audio/recorder.rs`, docs)

## Tests
- [x] Add tests for transcript queue drop/flush behavior. (`rust_tui/src/bin/codex_overlay.rs`)
- [x] Add tests for mic meter calculations. (`rust_tui/src/mic_meter.rs`)
- [x] Add tests for `stt.rs` (at least error paths). (`rust_tui/src/stt.rs`)
- [x] Add tests for `ui.rs` (input handling/rendering invariants). (`rust_tui/src/ui.rs`)
- [x] Add tests for VAD capture logic (record_with_vad path). (`rust_tui/src/audio/recorder.rs`)
- [x] Add tests for log gating defaults (logs off by default, opt-in flags/env). (`rust_tui/src/bin/codex_overlay.rs`, `rust_tui/src/app.rs`)
- [x] Add config validation tests for mic-meter duration bounds. (`rust_tui/src/config.rs`)
- [x] Add tests for `CLAUDE_CMD` sanitization. (`rust_tui/src/ipc.rs`, `rust_tui/src/config.rs`)

## Security and config
- [x] Make `--dangerously-skip-permissions` configurable (not hardcoded). (`rust_tui/src/ipc.rs`)
- [x] Sanitize `CLAUDE_CMD` env var with existing binary validator. (`rust_tui/src/ipc.rs`)
- [x] Align `whisper-rs` and `whisper-rs-sys` versions. (`rust_tui/Cargo.toml`)

## Docs and versioning
- [x] Fix CHANGELOG references to missing `docs/architecture/...` paths. (`dev/CHANGELOG.md`)
- [x] Sync version numbers between CHANGELOG and Cargo.toml.
- [x] Update README project structure to include `mic_meter.rs` and `audio/recorder.rs`. (`README.md`)
- [x] Fix or deprecate `ts_cli/README.md` (missing `package.json` makes instructions non-runnable).

## Whisper and STT follow-ups
- [x] Validate mic-meter durations to avoid huge in-memory buffers. (`rust_tui/src/config.rs`)
- [x] Replace `buffer.lock().unwrap()` panic in `record_for` with error handling. (`rust_tui/src/audio/recorder.rs`)
- [x] Evaluate default silence tail of 3000ms; consider reducing to 500-1000ms if it hurts latency. (`rust_tui/src/config.rs`)
- [x] Consider reducing mutex work in audio callbacks (lock-free ring buffer or pre-allocated channels). (`rust_tui/src/audio/recorder.rs`)
- [x] Document non-streaming STT behavior and lack of chunk overlap (if desired). (docs)
- [x] Add optional language auto-detect mode for Whisper (config + docs). (`rust_tui/src/stt.rs`, `rust_tui/src/config.rs`)
- [x] Expose Whisper params (beam size / temperature) for accuracy tuning. (`rust_tui/src/stt.rs`, `rust_tui/src/config.rs`)
- [x] Consider simple VAD smoothing to reduce flapping in noise. (`rust_tui/src/audio.rs`)
