# Architecture Notes — 2025-11-12

*Previous day: [`docs/architecture/2025-11-11/`](../2025-11-11/ARCHITECTURE.md)*

## Summary
- Bootstrapped the governance scaffolding required by `agents.md`: created the repository `CHANGELOG.md`, root `PROJECT_OVERVIEW.md`, and the initial daily architecture folder (`docs/architecture/2025-11-12/`).
- Documented the navigation contract via `master_index.md`, ensuring a single index file points at major directories and governance artifacts.
- Consolidated legacy documentation (root `architecture.md`, developer/testing guides, Claude audit) into the standardized folder structure (`docs/architecture/YYYY-MM-DD/`, `docs/guides/`, `docs/audits/`).
- Refreshed living guides (`architecture_overview.md`, `master_doc.md`, `HOW_TO_TEST_AUDIO.md`, `plan.md`) to match the current Rust pipeline after verifying `cargo --version` and the documented commands.
- Added a repository `README.md`, implemented the “You Are Here” section in `PROJECT_OVERVIEW.md`, and migrated the reusable guides into `docs/references/` while archiving obsolete copies under `docs/archive/OBSOLETE_GUIDES_2025-11-12/`.
- Extended `agents.md` with an end-of-session checklist so every workday records architecture notes, changelog updates, and the refreshed “You Are Here” block.
- Consolidated all CI/CD references into the single `docs/references/cicd_plan.md`, folding in the old dependencies + implementation plan content and archiving the redundant files under `docs/archive/OBSOLETE_REFERENCES_2025-11-12/`.
- Expanded the CI/CD plan with appendices (phase playbooks, tooling/dependency matrix, ops & troubleshooting) so the new canonical file captures all 1,500+ lines of historical detail without scattering sources.
- Adopted and logged the latency remediation plan (`docs/audits/latency_remediation_plan_2025-11-12.md`), updated `PROJECT_OVERVIEW.md` to prioritize latency stabilization above other workstreams, and linked the audit plan for future CI gating.
- Documented the Phase 2A ➜ 2B ➜ 3 ladder (incremental silence-stop, chosen chunked overlap, future async) plus the detailed backpressure/frame/shutdown/fallback policies that make Phase 2B production-ready.
- Enriched the latency plan with failure-mode hierarchy, VAD config defaults, explicit resource budgets, observability metrics, CI enforcement hooks, and a 15-day execution timeline so engineering + CI have one canonical specification.
- Added state-machine + lifecycle requirements, config surface/deployment profile expectations, and concurrency limits to the latency plan so future work cannot regress into ambiguous worker management.
- Captured the readiness audit (`docs/audits/READINESS_AUDIT_2025-11-12.md`) findings so execution order now follows the audit’s gating recommendations.

## Readiness Audit Summary
- Governance artifacts are healthy, but the implementation is still “Phase 1” (fixed-duration capture + serial STT).
- Phase 2A (silence-aware capture) must be designed/implemented before the Phase 2B chunked overlap described in the latency plan.
- Python fallback must move behind a dev-only feature flag; release/CI builds must error instead of silently running Python.
- perf_smoke + memory_guard CI jobs are still missing; these must be in place before declaring latency work “done.”
- `app.rs` and `pty_session.rs` violate the 200–300 LOC guideline; decomposition planning must happen alongside Phase 2A.

## Phase 2A Design (Silence-Aware Capture)
**Goal**: Replace `thread::sleep(seconds)` capture with VAD-driven early stop while keeping transcription serial. This delivers immediate latency wins and unblocks Phase 2B.

**Options**:
1. *Earshot* (pure Rust VAD)
   - Pros: no FFI toolchain, easy to tune, MIT license.
   - Cons: slightly higher CPU cost than WebRTC VAD.
2. *webrtc-vad-sys* (C binding)
   - Pros: industry-proven algorithm, deterministic output.
   - Cons: requires C toolchain, more complex builds across macOS/Linux/Windows.
3. *Custom energy threshold*
   - Pros: trivial to implement.
   - Cons: extremely sensitive to background noise; not acceptable for production.

**Recommendation**: Adopt **Earshot** for Phase 2A. It keeps the build pure Rust and meets our latency/maintenance goals while leaving room to swap to WebRTC VAD later if needed.

**Design Points**:
- Frame size: 20 ms @ 16 kHz mono (320 samples). Maintain a small ring buffer per callback invocation.
- Termination rule: stop when trailing silence ≥ `voice.silence_tail_ms` or when `voice.max_capture_ms` hard cap hits.
- Preserve last `voice.lookback_ms` (~500 ms) prior to silence to avoid truncating final syllables.
- Metrics: log `speech_ms`, `silence_tail_ms`, `frames_processed`, `early_stop_reason` per utterance.
- Config additions (defaults shown):
  - `voice.max_capture_ms = 10000`
  - `voice.silence_tail_ms = 500`
  - `voice.min_speech_ms_before_stt = 300`
  - `voice.vad_threshold_db = -40`
  - `voice.vad_frame_ms = 20`

**Testing / Validation**:
- Unit tests with synthetic clips (speech + silence) verifying stop time and minimum speech guard.
- Integration test comparing baseline vs Phase 2A latency on golden clips (store results in logs + docs).
- Edge cases: constant noise, silent input, manual stop (Ctrl+R) ensures resources clean up.

**Exit Criteria for Phase 2A**:
1. Earshot-driven capture replaces `sleep(seconds)`; CLI `--seconds` becomes a max bound.
2. Config + metrics in place; perf_smoke job consumes the new metrics.
3. Python fallback compile-time gated (`python_fallback` feature disabled in release/CI, optional in dev).
4. Phase 2A latency benchmark recorded under `docs/architecture/YYYY-MM-DD/` with before/after numbers.

## Immediate Priorities
1. **Implement Phase 2A** (Earshot VAD, config surface, metrics, tests).
2. **Feature-gate Python fallback** (dev-only; release/CI error instead of fallback).
3. **Add perf_smoke + memory_guard workflows** per latency plan; perf_smoke must fail when P95 ≥ 750 ms or total ≥ 10 s on CI hardware.
4. **Draft module decomposition plan** for `app.rs` and `pty_session.rs` (options + approvals before coding).

## Decisions
1. **Daily Architecture Structure**
   - One folder per working day under `docs/architecture/YYYY-MM-DD/`, each containing `ARCHITECTURE.md`, daily `CHANGELOG.md`, and any supplemental diagrams/files.
   - Every folder links back to the previous day to provide traceable breadcrumbs.
2. **Project Overview Scope**
   - `PROJECT_OVERVIEW.md` serves as the canonical roadmap: mission, current focus, architectural decision log pointer.
   - Updates occur whenever mission/focus shifts or when a new daily folder becomes "latest."
3. **Documentation Consolidation**
   - All architecture content now resides inside dated folders; the legacy overview has been migrated to `docs/architecture/2025-11-11/`.
   - Reference guides live under `docs/references/` (formerly `docs/guides/`) and audits under `docs/audits/` so the root remains uncluttered.
4. **Guide Accuracy Sweep**
   - Created the new `docs/references/` directory with `quick_start.md`, `testing.md`, `python_legacy.md`, `milestones.md`, and `troubleshooting.md`, and annotated the legacy plan in `docs/archive/MVP_PLAN_2024.md`.
   - Logged the audit updates here and in the repository `CHANGELOG.md` to keep traceability intact.

## Alternatives Considered
- **Single rolling architecture document**: rejected because it conflicts with the daily traceability requirement and makes CI enforcement harder.
- **Weekly aggregation only**: rejected for now; will revisit once the daily cadence proves burdensome (tracked in `agents.md` improvement notes).

## Benchmarks / Metrics
- Governance scaffolding only; no runtime measurements taken today. Latency/benchmark instrumentation will be added in subsequent phases.

## Next Steps
1. Finalize Phase 2A requirements (Earshot VAD design, config surface) and begin implementation.
2. Add `perf_smoke` + `memory_guard` workflows to CI so latency/memory SLAs gate merges.
3. Feature-gate Python fallback (dev-only feature, disabled in release/CI) and document the behavior change.
4. Draft module decomposition options for `app.rs` and `pty_session.rs` (≤300 LOC target) for approval.

See `docs/audits/latency_remediation_plan_2025-11-12.md` for the complete latency specification.

Phase 2A design (Earshot VAD, config, metrics) is now approved; implementation begins next session.
