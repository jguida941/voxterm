# Master Plan (Active)

## Purpose
- Single source of truth for active work, audits, and verification.
- New items go here; completed items move to `dev/archive/` with a dated entry.
- Roadmaps/design docs (e.g., UI enhancement plan) are reference-only unless an item is explicitly added here.

## References
- `agents.md` (SDLC policy and verification)
- `dev/ARCHITECTURE.md`
- `dev/DEVELOPMENT.md`
- `dev/adr/`
- `dev/CHANGELOG.md`

## Active work

### P0 - Reliability and UX correctness
- [x] MP-001 Fix transcript queue flush reliability (user-reported).
- [x] MP-002 Stop status spam: repeated "Transcript ready (Rust)" lines.
- [x] MP-003 Investigate unexpected "Use /skills..." output in UI and confirm source (not present in repo; likely upstream CLI output).
- [x] MP-004 Confirm auto-voice recording visibility (REC tag + meter) while capture is active (added full-mode banner test coverage).
- [x] MP-005 Fix Homebrew formula: update `rust_tui/Cargo.toml` → `src/Cargo.toml` in homebrew-voxterm repo.
- [x] MP-041 Align minimal HUD with CLI style: single-line strip (no borders/background) showing mode + status (AUTO/MANUAL/REC/PROC + dB when recording).
- [x] MP-042 Hidden HUD should never overlap CLI output (reserve a blank row when idle; indicator only while recording/processing).
- [x] MP-043 Add hotkey to cycle HUD style (Full ↔ Minimal ↔ Hidden) with status toast.
- [x] MP-044 Auto-assign default theme by backend when user does not specify `--theme`.

### P1 - Testing and verification
- [x] MP-010 Add integration tests for voice -> STT -> injection flow.
- [x] MP-011 Add PTY crash recovery tests (spawn/child death handling).
- [x] MP-012 Add integration tests for IPC session event loop behavior.
- [x] MP-013 Add concurrency stress tests for multi-threaded audio capture paths.
- [x] MP-014 Run mutation testing locally and record score (4.41% on 2026-02-02; below 80% threshold).
- [ ] MP-015 Improve mutation score by adding targeted tests for high-value paths.
- [ ] MP-016 Stress test under heavy I/O to confirm bounded memory behavior.
- [x] MP-017 Manual QA checklist: auto-voice status visibility, queue flush in insert/auto modes, prompt log off by default, two terminals running independently.
- [x] MP-018 Verify CSI-u filtering fix in real sessions.
- [x] MP-019 Verify transcript queueing while Codex is busy (waits for next prompt).

### P1 - Safety and code clarity
- [x] MP-020 Add SAFETY comments for unsafe blocks in PTY and other unsafe sections.
- [x] MP-021 Add module-level docs and public API docs for core modules (voice, audio, pty_session).
- [x] MP-022 Add struct/field docs where purpose is non-obvious and document complex logic blocks.
- [x] MP-023 Extract magic numbers to `config/defaults.rs` or named constants.
- [x] MP-024 Remove or justify dead code (`manual_stop`).
- [x] MP-025 Standardize error message format where inconsistent.
- [x] MP-026 Add PTY child_exec error diagnostics (capture errno for failed syscalls).
- [x] MP-027 Address stderr redirection race during model load (or document as acceptable).
- [x] MP-028 Remove unnecessary clone in voice metrics message path.
- [x] MP-045 Optimize `audio_meter::format_waveform` to avoid per-frame allocations.
- [x] MP-046 Reduce status update cloning (pre-allocate meter_levels Vec with METER_HISTORY_MAX capacity).
- [x] MP-047 Log writer I/O failures (flush/write) with context to aid debugging.
- [x] MP-048 Consolidate status-line formatting helpers to reduce duplication and improve maintainability.
- [x] MP-049 Group writer thread state into a struct to simplify redraw logic.
- [x] MP-050 Reduce oversized parameter lists (e.g., `handle_voice_message`) with context structs.
- [x] MP-056 Add a pre-refactor docs readiness checklist (README/QUICK_START/USAGE/CLI_FLAGS/INSTALL/TROUBLESHOOTING + screenshots).
- [x] MP-058 Fix buffer bounds panic in CSI-u sequence parsing (validate length before indexing).
- [x] MP-059 Add `#[inline]` hints to hot-path functions (display_width, level_color, rms_db, peak_db).
- [x] MP-060 Add `#[must_use]` attributes to key struct/function returns.
- [x] MP-061 Optimize hot-path formatters to use push_str instead of format! macros.
- [x] MP-062 Execute modularization plan for `main.rs` + large modules (see `dev/active/MODULARIZATION_PLAN.md`).
- [x] MP-063 Naming clarity pass for multi-backend readability (see `dev/active/MODULARIZATION_PLAN.md`, Track G).
- [x] MP-064 Legacy TUI naming alignment (`legacy_tui/`, `legacy_ui.rs`, `run_legacy_ui`, `CodexApp`) + doc references.
- [x] MP-065 Resolve module naming overlap (`backend` registry vs legacy `codex` backend) and document/rename as needed.
- [x] MP-066 Add overlay `--login` preflight to run backend CLI authentication (Codex/Claude) before startup.
- [x] MP-067 Fix `--theme` default handling so backend defaults apply when the flag is not set (align clap defaults + add test).
- [x] MP-068 Replace runtime `.lock().unwrap()` with a safe lock helper that logs and recovers from poisoned mutexes.
- [x] MP-070 Disambiguate Codex runtime naming from provider registry (`CliBackend` → `CodexCliBackend`, `CodexBackend` trait → `CodexJobRunner`, `BackendEvent/Job` → `CodexEvent/Job`), update IPC + legacy TUI imports.
- [x] MP-071 Rename codex-overlay test artifacts (`tests/codex_overlay_cli.rs`) and any lingering "Codex overlay" references to `voxterm` for clarity.

### P1 - Architecture decision tracking
- [x] MP-038 Draft ADRs for upcoming UI enhancement architecture (focus/selection model, SelectableMenu reuse, preferences + migrations, action registry + keybindings, history storage, render guarantees).

### P1 - Themes and branding
- [x] MP-051 Add Claude theme palette based on Anthropic brand colors.
- [x] MP-052 Define a Codex default palette (OpenAI-style neutral or user-provided) without claiming official brand colors.
- [x] MP-053 Document backend→theme defaults and how users override via `--theme`.
- [x] MP-057 Add ChatGPT theme (`--theme chatgpt`) with emerald green brand color.

### P2 - Observability, performance, and UX improvements
- [x] MP-030 Add structured logging (tracing) for better diagnostics.
- [ ] MP-031 Add PTY health monitoring to detect hung processes.
- [ ] MP-032 Retry logic for transient audio device failures.
- [ ] MP-033 Add benchmarks to CI for latency regression detection.
- [ ] MP-034 Add mic-meter hotkey for calibration.
- [x] MP-035 Optional HUD input preview while Codex is thinking. (Dropped; not needed)
- [x] MP-036 Investigate processing delay/freeze after sending input while Codex is thinking (after queue fix). (Resolved)
- [ ] MP-037 Consider making PTY output channel capacity configurable.
- [x] MP-040 Add settings overlay with arrow-key navigation and button-style controls.
- [x] MP-069 IPC TODOs: add PTY-backed output streaming in IPC mode and track real job duration metrics.

# Optional dont do yet
- [ ] MP-054 Add optional right-panel visualization modes (Ribbon/Dots/Chips) to minimal HUD strip.
- [ ] MP-055 Add quick theme switcher in settings (recent themes + backend defaults).

## Archive log
- `dev/archive/2026-01-29-claudeaudit-completed.md`
- `dev/archive/2026-01-29-docs-governance.md`
- `dev/archive/2026-02-01-terminal-restore-guard.md`
- `dev/archive/2026-02-01-transcript-queue-flush.md`
- `dev/archive/2026-02-02-release-audit-completed.md`
