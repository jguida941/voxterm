# Production Readiness Audit - VoxTerm Rust TUI (Verified)

Date: 2026-01-29
Version: Cargo.toml = 1.0.13 (CHANGELOG has 1.0.14 entry)
Auditor: Verified via static code inspection

Scope
- Rust TUI + overlay (`rust_tui/src`)
- Docs + TS CLI docs

Summary (verified)
- Core overlay/TUI architecture is solid, but several production risks remain.
- Main risks: unbounded buffers/channels (overlay + input + combined_raw), prompt logging always on/unbounded, IPC permissions flag hardcoded, missing tests (stt/ui), and doc/version drift.

Checklist (all audit issues to resolve)

Memory & Buffering
- [x] Bound overlay input/writer channels or add backpressure/drop policy. (`rust_tui/src/bin/codex_overlay.rs`)
- [x] Add max length cap for TUI input buffer. (`rust_tui/src/app.rs`)
- [x] Cap `combined_raw` growth in PTY call. (`rust_tui/src/codex.rs`)
- [x] Make voice job channel bounded or otherwise cap growth. (`rust_tui/src/voice.rs`)
- [x] Surface VAD frame drops to the user when non-zero. (`rust_tui/src/audio/recorder.rs`)

Logging / Disk / Privacy
- [x] Default all logging OFF in production; enable only via explicit flags/env (dev mode). (global logging gate)
- [x] Make prompt log opt-in (flag/env) and disable by default. (`rust_tui/src/bin/codex_overlay.rs`)
- [x] Add runtime size cap/rotation for prompt log. (`rust_tui/src/bin/codex_overlay.rs`)
- [x] Add runtime size cap/rotation for debug log, and avoid heavy per-write overhead. (`rust_tui/src/app.rs`)
- [x] Remove or gate partial prompt/response logging in IPC (`prompt[..30]`, `line[..50]`). (`rust_tui/src/ipc.rs`)
- [x] Add a `--no-logs` or equivalent privacy flag to disable content logging. (docs + code)

Behavior / UX
- [x] Warn when transcript queue drops the oldest item. (`rust_tui/src/bin/codex_overlay.rs`)
- [x] Fix insert-mode queue stall (only advance `last_enter_at` when newline sent). (`rust_tui/src/bin/codex_overlay.rs`)
- [x] Keep auto-voice status visible while auto mode is enabled and actively listening. (status line)
- [ ] Stop status spam: repeated “Transcript ready (Rust pipeline)” lines. (backlog issue)
- [ ] Investigate unexpected “Use /skills…” output showing up in the UI. (backlog issue)
- [x] Add platform-specific mic permission guidance in error messages. (`rust_tui/src/audio/recorder.rs`, docs)
- [x] Add device hotplug/recovery or document lack of hotplug support. (`rust_tui/src/audio/recorder.rs`, docs)

Tests
- [x] Add tests for transcript queue drop/flush behavior. (`rust_tui/src/bin/codex_overlay.rs`)
- [x] Add tests for mic meter calculations. (`rust_tui/src/mic_meter.rs`)
- [x] Add tests for `stt.rs` (at least error paths). (`rust_tui/src/stt.rs`)
- [x] Add tests for `ui.rs` (input handling/rendering invariants). (`rust_tui/src/ui.rs`)
- [x] Add tests for VAD capture logic (record_with_vad path). (`rust_tui/src/audio/recorder.rs`)
- [x] Add tests for log gating defaults (logs off by default, opt-in flags/env). (`rust_tui/src/bin/codex_overlay.rs`, `rust_tui/src/app.rs`)
- [x] Add config validation tests for mic-meter duration bounds. (`rust_tui/src/config.rs`)
- [x] Add tests for `CLAUDE_CMD` sanitization. (`rust_tui/src/ipc.rs`, `rust_tui/src/config.rs`)

Security / Config
- [x] Make `--dangerously-skip-permissions` configurable (not hardcoded). (`rust_tui/src/ipc.rs`)
- [x] Sanitize `CLAUDE_CMD` env var with existing binary validator. (`rust_tui/src/ipc.rs`)
- [x] Align `whisper-rs` and `whisper-rs-sys` versions. (`rust_tui/Cargo.toml`)

Docs / Versioning
- [x] Fix CHANGELOG references to missing `docs/architecture/...` paths. (`dev/CHANGELOG.md`)
- [x] Sync version numbers between CHANGELOG and Cargo.toml.
- [x] Update README project structure to include `mic_meter.rs` and `audio/recorder.rs`. (`README.md`)
- [x] Fix or deprecate `ts_cli/README.md` (missing `package.json` makes instructions non-runnable).

Whisper/STT-specific risks
- [x] Validate mic-meter durations to avoid huge in-memory buffers. (`rust_tui/src/config.rs`)
- [x] Replace `buffer.lock().unwrap()` panic in `record_for` with error handling. (`rust_tui/src/audio/recorder.rs`)
- [x] Evaluate default silence tail of 3000ms; consider reducing to 500–1000ms if it hurts latency. (`rust_tui/src/config.rs`)
- [x] Consider reducing mutex work in audio callbacks (lock-free ring buffer or pre-allocated channels). (`rust_tui/src/audio/recorder.rs`)
- [x] Document non-streaming STT behavior and lack of chunk overlap (if desired). (docs)
- [x] Add optional language auto-detect mode for Whisper (config + docs). (`rust_tui/src/stt.rs`, `rust_tui/src/config.rs`)
- [x] Expose Whisper params (beam size / temperature) for accuracy tuning. (`rust_tui/src/stt.rs`, `rust_tui/src/config.rs`)
- [x] Consider simple VAD smoothing to reduce flapping in noise. (`rust_tui/src/audio.rs`)

Runtime verification
- [x] Re-run clippy after changes.
- [ ] Re-run mutation testing and record score.
- [ ] Stress-test under heavy I/O to confirm no runaway memory growth.
- [ ] Manual QA: auto-voice status remains visible while listening; queue flush works in insert/auto modes; prompt log off by default; two terminals can run independently.

Confirmed Findings

Memory & Buffering
- Output scrollback capped at 500 lines. (`rust_tui/src/app.rs`)
- Backend event queue bounded at 1024. (`rust_tui/src/codex.rs`)
- PTY output channel bounded at 100. (`rust_tui/src/pty_session.rs`)
- Overlay input/writer channels are unbounded, so slow stdout can accumulate memory. (`rust_tui/src/bin/codex_overlay.rs`)
- TUI input buffer (`App.input`) is unbounded. (`rust_tui/src/app.rs`)
- `combined_raw` in Codex PTY call grows until timeout (no size cap). (`rust_tui/src/codex.rs`)
- Voice job uses unbounded `std::sync::mpsc::channel()`. (`rust_tui/src/voice.rs`)

Logging / Disk
- Debug log prunes only on startup if >5MB; no runtime rotation. (`rust_tui/src/app.rs`)
- Debug log opens/appends per write; no size cap. (`rust_tui/src/app.rs`)
- Prompt log is always enabled (default temp path) and appends without size cap. (`rust_tui/src/bin/codex_overlay.rs`)

Behavior / UX
- Pending transcript queue drops the oldest item when full; no user-visible drop warning. (`rust_tui/src/bin/codex_overlay.rs`)
- Insert mode can stall queued transcripts until a new prompt appears (prompt gating uses `last_enter_at`). (`rust_tui/src/bin/codex_overlay.rs`)

Tests
- `stt.rs` has no tests. (`rust_tui/src/stt.rs`)
- `ui.rs` has no tests. (`rust_tui/src/ui.rs`)
- `audio/recorder.rs` VAD path is stubbed under `cfg(test)`. (`rust_tui/src/audio/recorder.rs`)
- No tests for transcript queue drop/flush or mic-meter calculations.

Security / Config
- `--dangerously-skip-permissions` is hardcoded for IPC auth. (`rust_tui/src/ipc.rs`)
- `CLAUDE_CMD` env var is used without validation/sanitization. (`rust_tui/src/ipc.rs`)
- `whisper-rs` / `whisper-rs-sys` version mismatch. (`rust_tui/Cargo.toml`)

Documentation / Versioning
- `dev/CHANGELOG.md` references `docs/architecture/...` paths that do not exist in repo.
- Version mismatch: `rust_tui/Cargo.toml` 1.0.13 vs CHANGELOG 1.0.14.
- README project structure omits `mic_meter.rs` and `audio/recorder.rs`. (`README.md`)
- `ts_cli/README.md` instructions are not runnable as written (no `package.json` in repo).

Removed or Corrected Claims
- Removed incorrect "default audio channel 64"; AppConfig default is 100.
- Removed claim that bounded channels prevent memory exhaustion; overlay channels are unbounded.
- Removed "fragile unwrap on buffer.last()"; guarded by length check.
- Removed unverified items (clippy warnings, mutation score, exact unsafe count, TS CLI runtime status beyond missing package.json).

Items Requiring Runtime Verification (not asserted)
- Mutation score / missed mutants in current run.
- Clippy warnings after recent changes.
- Performance characteristics under extreme I/O load.

Whisper/STT Risk Audit (code review)

Audio format correctness
- CPAL sample conversions are correct: f32 passthrough, i16 scaled by 1/32768, u16 centered and scaled; downmix averages channels; resample targets 16 kHz. (`rust_tui/src/audio/recorder.rs`, `rust_tui/src/audio.rs`)
- VAD pipeline converts device-rate frames to target sample rate before VAD and ensures frame length. (`rust_tui/src/audio/recorder.rs`)
- Risk: mic meter durations have no validation; `record_for` buffers the full duration, so huge values can cause large memory use. (`rust_tui/src/config.rs`, `rust_tui/src/audio/recorder.rs`)

Buffering & chunking
- VAD path uses bounded channels with drop counts and a capped `FrameAccumulator` based on `voice_buffer_ms`. (`rust_tui/src/audio/recorder.rs`, `rust_tui/src/audio.rs`)
- Drops are tracked but not surfaced to users; sustained overload can degrade transcripts silently. (`rust_tui/src/audio/recorder.rs`)
- No chunk overlap because STT runs on full captured clip (non-streaming); acceptable for this design.

Performance & resource usage
- Whisper model loads once and is reused; not reloaded per capture. (`rust_tui/src/app.rs`, `rust_tui/src/bin/codex_overlay.rs`)
- Transcription runs on a worker thread; UI thread remains responsive. (`rust_tui/src/voice.rs`)
- STT uses up to `min(num_cpus, 8)` threads; avoids full-core saturation but can still be heavy on low-end machines. (`rust_tui/src/stt.rs`)

Concurrency & deadlocks
- No async/await in voice pipeline; no locks held across awaits. (`rust_tui/src/voice.rs`)
- Audio callbacks lock a mutex for buffering/dispatch; currently no contending holders, but this is a realtime callback risk if future code adds contention. (`rust_tui/src/audio/recorder.rs`)

Panic paths in audio/STT
- `record_for` uses `buffer.lock().unwrap()` after capture; a poisoned lock would panic. Consider `map_err` instead. (`rust_tui/src/audio/recorder.rs`)

FFI / unsafe usage
- Whisper access goes through `whisper_rs` with a single `unsafe` log callback and libc `dup/dup2` for stderr suppression; the transcriber is guarded by `Arc<Mutex>` to avoid concurrent access. (`rust_tui/src/stt.rs`)

Privacy & data leakage
- Prompt logger records prompt lines and is always enabled by default; this can capture sensitive content and writes indefinitely. (`rust_tui/src/bin/codex_overlay.rs`)
- Regular debug logs do not include transcript text; latency benchmark binary prints transcripts to stdout (dev-only). (`rust_tui/src/bin/latency_measurement.rs`)

Device/compatibility risks
- Uses default input device; no hotplug re-init. Device disconnects yield errors and may fall back to Python if allowed. (`rust_tui/src/audio/recorder.rs`, `rust_tui/src/voice.rs`)
