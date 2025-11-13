# Project Overview

## 🎯 You Are Here

- **Current Session**: 2025-11-13
- **Latest Notes**: [`docs/architecture/2025-11-13/`](docs/architecture/2025-11-13/)

**Today We Finished**
- Opened the 2025-11-13 daily folder and logged the formal Phase 2A kickoff plus Earshot VAD approval.
- Captured Phase 2A exit criteria (config surface, metrics, tests, CI hooks) to gate Phase 2B readiness.
- Reaffirmed documentation/governance updates (`master_index.md`, `PROJECT_OVERVIEW.md`) pointing to the latest notes.
- Landed the first Phase 2A code drop: `VadEngine` trait + Earshot feature gate + voice-pipeline wiring that now emits per-utterance metrics for future perf_smoke gating.
- Shipped the async Codex worker (new `codex.rs`, `App::poll_codex_job`, spinner/cancel UX, telemetry, and unit tests), so the TUI stays responsive while Codex runs and Phase 2A work can resume.

**In Progress**
- Latency stabilization Phase 2A (Earshot-based VAD + early stop, config surface, metrics) — prerequisite for Phase 2B
- CI perf/memory gating (`perf_smoke`, `memory_guard`) and documentation enforcement checks
- Defining CI/doc lint checks for daily folder & changelog enforcement
- Planning the `app.rs`/`pty_session.rs` decomposition (pending design options)
- Tracking Codex worker telemetry in CI (add perf_smoke hook) now that the async path is live; any regressions should fail the pipeline.

**Next Session**
1. Resume Phase 2A execution: finish Earshot VAD integration, early-stop logic, config keys, and latency benchmarks now that Codex calls no longer block the UI.
2. Wire CI guards (`perf_smoke`, `memory_guard`, docs-check) so latency + documentation policies gate merges (including the new `timing|phase=codex_job` metrics).
3. Feature-gate Python fallback (dev-only feature flag) and document UX/error handling.
4. Draft design options for splitting `app.rs`/`pty_session.rs` into ≤300 LOC modules for approval before implementation.

---

Central roadmap for the Codex Voice-to-Codex wrapper. This document summarizes the current scope, major architectural decisions, and pointers into the dated architecture log.

## Mission
- Deliver a Rust-first voice interface that extends (not replaces) Codex, adding voice capture, standardized formatting, diagnostics, CI/testing harnesses, and future IDE-style tooling.
- Enforce the SDLC discipline defined in `agents.md`: design-first workflow, modular code (200–300 LOC per module), full traceability (CHANGELOG + daily architecture notes), and strong test/benchmark coverage.

## Current Focus
1. Stand up mandatory governance scaffolding (daily architecture folders, CHANGELOG, master index).
2. Re-architecture the voice pipeline to meet sub-second round-trip targets (silence-aware capture + overlapped STT).
3. Decompose oversized modules (e.g., `rust_tui/src/app.rs`) while preserving Codex integration surface.

## Architectural Decision Log
- Latest daily notes: [`docs/architecture/2025-11-13/`](docs/architecture/2025-11-13/)
- Previous day baseline: [`docs/architecture/2025-11-12/`](docs/architecture/2025-11-12/)
- Historical decisions: see `docs/architecture/` index (one folder per working day; each folder links back to the previous day to form a breadcrumb trail).

## Directory Map
See `master_index.md` for a detailed description of every top-level directory and governance file.

## Key Reference Docs
- `docs/references/cicd_plan.md` — Single-source CI/CD blueprint covering the pipeline architecture, dependencies, and the phased implementation guide.
- `docs/references/quick_start.md` / `testing.md` — Operational procedures for building, running, and validating the voice wrapper.

## Update Checklist
When making changes:
1. Draft design reasoning + alternatives in the current day’s `docs/architecture/YYYY-MM-DD/ARCHITECTURE.md`.
2. Record user-approved decisions and benchmarks in the same folder.
3. Update this overview when major goals or decision links change.
4. Append the repository `CHANGELOG.md`.
5. Ensure CI sees both the daily architecture note and CHANGELOG updates.
