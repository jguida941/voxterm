# Master Plan (Active)

## Purpose
- Single source of truth for active work, audits, and verification.
- New items go here; completed items move to `dev/archive/` with a dated entry.
- Roadmaps/design docs (e.g., UI enhancement plan) are reference-only unless an item is explicitly added here.

## References
- `agents.md` (SDLC policy and verification)
- `dev/active/UI_ENHANCEMENT_PLAN.md` (design/roadmap reference; active priorities must be tracked here)
- `dev/active/release_audit_Feb2nd.md` (code audit + phased action plan)
- `dev/ARCHITECTURE.md`
- `dev/DEVELOPMENT.md`
- `dev/adr/`
- `dev/CHANGELOG.md`

## Source inputs
- `dev/active/BACKLOG.md` (migrated here)

## Active work

### P0 - Reliability and UX correctness
- [x] MP-001 Fix transcript queue flush reliability (user-reported).
- [ ] MP-002 Stop status spam: repeated "Transcript ready (Rust)" lines.
- [ ] MP-003 Investigate unexpected "Use /skills..." output in UI and confirm source.
- [ ] MP-004 Confirm auto-voice recording visibility (REC tag + meter) while capture is active.
- [x] MP-005 Fix Homebrew formula: update `rust_tui/Cargo.toml` → `src/Cargo.toml` in homebrew-voxterm repo.
- [x] MP-041 Align minimal HUD with CLI style: single-line strip (no borders/background) showing mode + status (AUTO/MANUAL/REC/PROC + dB when recording).
- [x] MP-042 Hidden HUD should never overlap CLI output (reserve a blank row when idle; indicator only while recording/processing).
- [x] MP-043 Add hotkey to cycle HUD style (Full ↔ Minimal ↔ Hidden) with status toast.
- [x] MP-044 Auto-assign default theme by backend when user does not specify `--theme`.

### P1 - Testing and verification
- [ ] MP-010 Add integration tests for voice -> STT -> injection flow.
- [ ] MP-011 Add PTY crash recovery tests (spawn/child death handling).
- [ ] MP-012 Add integration tests for IPC session event loop behavior.
- [ ] MP-013 Add concurrency stress tests for multi-threaded audio capture paths.
- [ ] MP-014 Run mutation testing locally and record score.
- [ ] MP-015 Improve mutation score by adding targeted tests for high-value paths.
- [ ] MP-016 Stress test under heavy I/O to confirm bounded memory behavior.
- [ ] MP-017 Manual QA checklist: auto-voice status visibility, queue flush in insert/auto modes, prompt log off by default, two terminals running independently.
- [ ] MP-018 Verify CSI-u filtering fix in real sessions.
- [ ] MP-019 Verify transcript queueing while Codex is busy (waits for next prompt).

### P1 - Safety and code clarity
- [ ] MP-020 Add SAFETY comments for unsafe blocks in PTY and other unsafe sections.
- [ ] MP-021 Add module-level docs and public API docs for core modules (voice, audio, pty_session).
- [ ] MP-022 Add struct/field docs where purpose is non-obvious and document complex logic blocks.
- [ ] MP-023 Extract magic numbers to `config/defaults.rs` or named constants.
- [ ] MP-024 Remove or justify dead code (`manual_stop`).
- [ ] MP-025 Standardize error message format where inconsistent.
- [ ] MP-026 Add PTY child_exec error diagnostics (capture errno for failed syscalls).
- [ ] MP-027 Address stderr redirection race during model load (or document as acceptable).
- [ ] MP-028 Remove unnecessary clone in voice metrics message path.
- [ ] MP-045 Optimize `audio_meter::format_waveform` to avoid per-frame allocations.
- [ ] MP-046 Reduce status update cloning (avoid cloning full `StatusLineState` on every send).
- [ ] MP-047 Log writer I/O failures (flush/write) with context to aid debugging.
- [ ] MP-048 Consolidate status-line formatting helpers to reduce duplication and improve maintainability.
- [ ] MP-049 Group writer thread state into a struct to simplify redraw logic.
- [ ] MP-050 Reduce oversized parameter lists (e.g., `handle_voice_message`) with context structs.
- [ ] MP-056 Add a pre-refactor docs readiness checklist (README/QUICK_START/USAGE/CLI_FLAGS/INSTALL/TROUBLESHOOTING + screenshots).

### P1 - Architecture decision tracking
- [x] MP-038 Draft ADRs for upcoming UI enhancement architecture (focus/selection model, SelectableMenu reuse, preferences + migrations, action registry + keybindings, history storage, render guarantees).

### P1 - Themes and branding
- [x] MP-051 Add Claude theme palette based on Anthropic brand colors.
- [x] MP-052 Define a Codex default palette (OpenAI-style neutral or user-provided) without claiming official brand colors.
- [x] MP-053 Document backend→theme defaults and how users override via `--theme`.

### P2 - Observability, performance, and UX improvements
- [ ] MP-030 Add structured logging (tracing) for better diagnostics.
- [ ] MP-031 Add PTY health monitoring to detect hung processes.
- [ ] MP-032 Retry logic for transient audio device failures.
- [ ] MP-033 Add benchmarks to CI for latency regression detection.
- [ ] MP-034 Add mic-meter hotkey for calibration.
- [ ] MP-035 Optional HUD input preview while Codex is thinking.
- [ ] MP-036 Investigate processing delay/freeze after sending input while Codex is thinking (after queue fix).
- [ ] MP-037 Consider making PTY output channel capacity configurable.
- [x] MP-040 Add settings overlay with arrow-key navigation and button-style controls.
- [ ] MP-054 Add optional right-panel visualization modes (Ribbon/Dots/Chips) to minimal HUD strip.
- [ ] MP-055 Add quick theme switcher in settings (recent themes + backend defaults).

## Archive log
- `dev/archive/2026-01-29-claudeaudit-completed.md`
- `dev/archive/2026-01-29-docs-governance.md`
- `dev/archive/2026-02-01-terminal-restore-guard.md`
- `dev/archive/2026-02-01-transcript-queue-flush.md`
