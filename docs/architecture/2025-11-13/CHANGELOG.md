# Daily Changelog — 2025-11-13

## Added
- Created the 2025-11-13 architecture folder capturing the Phase 2A kickoff, Earshot approval, and success criteria checklist.
- Documented Phase 2A work plan (config surface, Earshot integration, metrics, tests, CI stubs) per the latency remediation plan.
- Implemented `VadEngine` interfaces in `rust_tui/src/audio.rs`, including the Earshot feature flag wiring and a `SimpleThresholdVad` fallback for non-Earshot builds.
- Added the `vad_earshot` feature and optional dependency placeholder in `rust_tui/Cargo.toml`.
- Introduced `rust_tui/src/vad_earshot.rs` (feature gated) plus the trait-based VAD factory used by `voice.rs`.
- Reworked `Recorder::record_with_vad` to prepare for chunked capture (non-test builds) and added per-utterance metrics scaffolding/logging.
- Updated `voice.rs` to call the new VAD-aware recorder path and emit capture metrics alongside transcripts.

## Pending
- Implementation of Earshot-based silence-aware capture and the accompanying metrics/tests (paused until Codex worker fix restores UI responsiveness).
- Addition of `perf_smoke` and `memory_guard` workflows tied to the new metrics.
- Async Codex worker module (`rust_tui/src/codex.rs`) plus UI changes in `app.rs`/`ui.rs` to prevent 30–60 second blocking when sending prompts.

## Notes
- Future updates to this file must capture concrete code/doc changes completed on 2025-11-13.
