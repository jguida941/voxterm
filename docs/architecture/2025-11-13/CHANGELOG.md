# Daily Changelog — 2025-11-13

## Added
- Created the 2025-11-13 architecture folder capturing the Phase 2A kickoff, Earshot approval, and success criteria checklist.
- Documented Phase 2A work plan (config surface, Earshot integration, metrics, tests, CI stubs) per the latency remediation plan.
- Implemented `VadEngine` interfaces in `rust_tui/src/audio.rs`, including the Earshot feature flag wiring and a `SimpleThresholdVad` fallback for non-Earshot builds.
- Added the `vad_earshot` feature and optional dependency placeholder in `rust_tui/Cargo.toml`.
- Introduced `rust_tui/src/vad_earshot.rs` (feature gated) plus the trait-based VAD factory used by `voice.rs`.
- Reworked `Recorder::record_with_vad` to prepare for chunked capture (non-test builds) and added per-utterance metrics scaffolding/logging.
- Updated `voice.rs` to call the new VAD-aware recorder path and emit capture metrics alongside transcripts.
- Captured the detailed Option 1 Codex worker design (state flow, cancellation, spinner plan, telemetry) in `ARCHITECTURE.md` so implementation can proceed under SDLC.
- Implemented the nonblocking Codex worker (`rust_tui/src/codex.rs` + `App::poll_codex_job`), spinner/cancel UX, and session handoff; TUI no longer freezes during 30–60 s Codex calls and cancellation now surfaces via Esc/Ctrl+C.
- Added unit tests for the worker success/error/cancel paths plus new UI-level tests that drive the spinner/cancel flow via the job hook harness; `cargo test --no-default-features` is now part of the daily verification until the `earshot` crate is reachable.

## Fixed
- Resolved Earshot API compilation errors (VoiceActivityProfile constants were already correct; cargo clean fixed caching issue).
- Applied `cargo fix` to remove unused imports and unnecessary `mut` qualifiers (4 warnings fixed).

## Pending
- Implementation of Earshot-based silence-aware capture and the accompanying metrics/tests.
- Addition of `perf_smoke` and `memory_guard` workflows tied to the new metrics.
- Manual testing of async Codex worker UI responsiveness and cancellation behavior.

## Notes
- Future updates to this file must capture concrete code/doc changes completed on 2025-11-13.
