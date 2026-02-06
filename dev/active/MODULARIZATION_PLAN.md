# VoxTerm Modularization Plan (Main + Large Modules)

## Overview

Refactor the largest files into focused modules to improve clarity, testability, and maintainability without changing behavior. The top priority remains extracting `main.rs` into a bootstrap + wiring file, then splitting other oversized modules (>400 lines) into dedicated submodules with clear responsibilities.

**Guiding constraints**
- No user-facing behavior changes unless explicitly planned.
- Keep all CLI flags, HUD behavior, and outputs stable.
- Prefer module directories (`foo/mod.rs` + submodules) for large files.
- Reduce parameter lists using context structs.
- Extract reusable helpers and constants into dedicated modules.
- Add unit tests for branch-heavy logic (especially control flow + parsing).

**Target size goals**
- `main.rs`: 400-600 lines.
- Large modules: 200-400 lines per file after split.
- No `#[allow(clippy::too_many_arguments)]` in hot paths.

---

## Baseline (largest files)

| File | Lines | Notes |
| --- | --- | --- |
| `src/bin/voxterm/main.rs` | 3366 | monolith; event loop + helpers |
| `src/bin/voxterm/status_line/` | 1875 | layout + formatting + state (split complete) |
| `src/bin/voxterm/prompt/` | 679 | regex + logger + tracker (split complete) |
| `src/bin/voxterm/voice_control/` | 674 | manager + pipeline + capture (split complete) |
| `src/bin/voxterm/writer/` | 629 | IO + redraw + overlay rendering (split complete) |
| `src/bin/voxterm/theme/` | 602 | palettes + borders + theme logic (split complete) |
| `src/bin/voxterm/input/` | 474 | parser + escape/mouse handling (split complete) |
| `src/bin/voxterm/config/` | 433 | CLI + backend resolution (split complete) |
| `src/bin/voxterm/audio_meter/` | 433 | measurement + rendering (split complete) |
| `src/bin/voxterm/transcript/` | 423 | queue + delivery + prompt gates (split complete) |
| `src/bin/voxterm/settings/` | 354 | settings state + render (split complete) |

---

## Phase 0: Prep + invariants

- Snapshot current behavior in notes (HUD styles, overlays, theme picker, settings interactions, voice capture flow).
- Establish quick regression checks:
  - `cargo test` baseline
  - `cargo build --release --bin voxterm`
- Record any known mutation gaps (see bottom).

---

## Track A: `main.rs` extraction (Phases 1-10)

### Phase 1: Terminal Management Module
**File:** `src/bin/voxterm/terminal.rs`
- Extract terminal sizing, reserved rows, SIGWINCH handling.
- Move tests for SIGWINCH and dimension helpers.

### Phase 2: Banner + CLI Utilities
**Files:** `src/bin/voxterm/banner.rs` (existing) + `src/bin/voxterm/cli_utils.rs` (new)
- Move banner helpers, device listing, stats printing guard.

### Phase 3: Overlay Presentation Module
**File:** `src/bin/voxterm/overlays.rs` (new)
- Move `OverlayMode` + overlay open/close helpers + overlay render wrappers.
- Add overlay dimension tests (width/height per mode).

### Phase 4: Arrow Key Parsing Module
**File:** `src/bin/voxterm/arrow_keys.rs`
- Extract arrow-key parsing + tests.

### Phase 5: Theme Ops Module
**File:** `src/bin/voxterm/theme_ops.rs`
- Extract theme cycling and picker selection helpers.
- Clarify overlap with `theme_picker.rs` (keep `theme_picker.rs` as rendering only).
**Status:** Done (helpers moved to `theme_ops.rs`).

### Phase 6: Settings Handlers Module
**File:** `src/bin/voxterm/settings_handlers.rs`
- Extract toggles/cycles and apply updates via context struct.
**Status:** Done (settings actions moved behind `SettingsActionContext`).

### Phase 7: Button Handlers Module
**File:** `src/bin/voxterm/button_handlers.rs`
- Extract button registry + focus + action handling with context struct.
**Status:** Done (button handling extracted to `button_handlers.rs`).

### Phase 8: Voice Message Drain
**File:** `src/bin/voxterm/voice_control/drain.rs`
- Move `drain_voice_messages` and helpers here.
**Status:** Done (drain moved into `voice_control/drain.rs`).

### Phase 9: Event Loop State + Context Consolidation
**File:** `src/bin/voxterm/event_state.rs`
- Create `EventLoopState` + `EventLoopDeps` + `EventLoopTimers`.
**Status:** Done (state/deps/timers extracted into `event_state.rs`).

### Phase 10: Event Loop Extraction
**File:** `src/bin/voxterm/event_loop.rs`
- Extract loop and handlers, keep `main.rs` as bootstrap + wiring only.
**Status:** Done (event loop moved into `event_loop.rs`).

---

## Track B: Status line modularization

### Phase 11: `status_line` module directory
**Target:** `src/bin/voxterm/status_line/`
- `mod.rs`: public API re-exports (`StatusLineState`, `StatusBanner`, `format_status_banner`, `status_banner_height`, `get_button_positions`).
- `state.rs`: enums + state structs + constants.
- `layout.rs`: responsive width breakpoints + panel sizing decisions.
- `format.rs`: row/section formatting helpers.
- `buttons.rs`: button layout + click positions + pill formatting.
- `animation.rs`: heartbeat + recording/processing frame helpers.
- `text.rs`: ANSI display width + truncation utilities.

**Tests**
- Banner height by width + HUD style.
- Button positions stable for minimal/full/hidden.
- Animation frame ranges.
**Status:** Done (status_line split into module directory).

---

## Track C: Writer modularization

### Phase 12: `writer` module directory
**Target:** `src/bin/voxterm/writer/`
- `mod.rs`: `WriterMessage`, `spawn_writer_thread`, public helpers.
- `state.rs`: `WriterState` + message handling.
- `render.rs`: status/overlay drawing + clear helpers.
- `mouse.rs`: mouse enable/disable and state.
- `sanitize.rs`: `sanitize_status` + `truncate_status`.

**Tests**
- Sanitization + truncation behavior.
- Rendering helpers with `Vec<u8>` writer to validate line counts.
**Status:** Done (writer split into module directory).

---

## Track D: Prompt + voice pipeline modularization

### Phase 13: `prompt` module directory
**Target:** `src/bin/voxterm/prompt/`
- `mod.rs`: re-exports.
- `regex.rs`: regex resolution + config.
- `logger.rs`: prompt log writer + rotation.
- `strip.rs`: ANSI stripping.
- `tracker.rs`: `PromptTracker` state machine.

**Tests**
- Regex override vs backend fallback.
- Log rotation thresholds.
- Prompt detection on partial/ANSI input.
**Status:** Done (prompt split into module directory).

### Phase 14: `voice_control` module directory
**Target:** `src/bin/voxterm/voice_control/`
- `mod.rs`: re-exports.
- `manager.rs`: `VoiceManager` + start/stop/poll.
- `pipeline.rs`: pipeline selection + fallback notes.
- `drain.rs`: `drain_voice_messages` + transcript preview + latency tracking.

**Tests**
- Pipeline selection based on recorder/transcriber availability.
- Drain path transitions for success/cancel/error.
**Status:** Done (voice_control split into module directory).

---

## Track E: Input + config + theme modularization

### Phase 15: `input` module directory
**Target:** `src/bin/voxterm/input/`
- `event.rs`: `InputEvent` enum.
- `parser.rs`: `InputParser` (escape parsing).
- `mouse.rs`: mouse click decoding.
- `spawn.rs`: input thread.
- Optionally fold `arrow_keys.rs` into `input/keys.rs` if cohesive.

**Tests**
- ESC sequence parsing (arrows, enter, mouse).
- CR/LF skip behavior.
**Status:** Done (input split into module directory).

### Phase 16: `config` module directory
**Target:** `src/bin/voxterm/config/`
- `cli.rs`: clap struct + enums.
- `backend.rs`: backend resolution.
- `theme.rs`: theme/color-mode resolution.
- `util.rs`: path parsing helpers.

**Tests**
- Backend resolution with shorthand flags.
- Theme selection with NO_COLOR + backend defaults.
**Status:** Done (config split into module directory).

### Phase 17: `theme` module directory
**Target:** `src/bin/voxterm/theme/`
- `mod.rs`: `Theme` enum + public API.
- `colors.rs`: `ThemeColors` + ANSI codes.
- `borders.rs`: `BorderSet` presets.
- `palettes.rs`: palette definitions (theme data).
- `detect.rs`: terminal detection helpers.

**Tests**
- Theme name parsing.
- Fallback for ANSI-only mode.
**Status:** Done (theme split into module directory).

---

## Track F: Audio, transcript, settings

### Phase 18: `audio_meter` module directory
**Target:** `src/bin/voxterm/audio_meter/`
- `mod.rs`: re-exports.
- `measure.rs`: RMS/peak + capture measurement.
- `format.rs`: meter/compact/waveform formatting.
- `recommend.rs`: threshold recommendations.

**Tests**
- RMS/peak output sanity.
- Waveform width handling.
**Status:** Done (audio_meter split into module directory).

### Phase 19: `transcript` module directory
**Target:** `src/bin/voxterm/transcript/`
- `session.rs`: `TranscriptSession` trait.
- `queue.rs`: pending transcript structs + merge.
- `delivery.rs`: send/flush logic.
- `idle.rs`: prompt readiness + gating.

**Tests**
- Queue merge ordering.
- Prompt-gated auto-send behavior.
**Status:** Done (transcript split into module directory).

### Phase 20: `settings` module directory
**Target:** `src/bin/voxterm/settings/`
- `state.rs`: `SettingsMenuState` + settings model.
- `render.rs`: overlay output builder.
- `items.rs`: control definitions + labels.

**Status:** Done (settings split into module directory).

**Tests**
- Render output row/width consistency.
- Menu navigation bounds.

---

## Track G: Naming + Directory Clarity (Readability Pass)

**Goal:** Make naming reflect multi-backend reality (Codex + Claude + Gemini) while keeping user-facing behavior stable. This is a readability-only pass: no feature changes, no behavior changes.

**Principles**
- Keep CLI flags, binary name, and external APIs stable.
- Prefer internal renames + type aliases for compatibility.
- Update docs and tests to match new names.
- Only change file paths when the benefits are clear and contained.

### Phase 21: Naming audit + rename map
- Inventory “codex” naming used in generic components.
- Produce a rename map that separates **provider-specific** vs **generic** concepts.
- Classify each candidate as: `Keep` (Codex-specific), `Rename` (generic), `Alias` (to avoid breaking).

**Initial rename map (draft):**
| Area | Current | Proposed | Class |
| --- | --- | --- | --- |
| PTY session | `PtyCodexSession` | `PtyCliSession` | Rename |
| PTY spawn | `spawn_codex_child` | `spawn_pty_child` | Rename (done) |
| Overlay dir | `src/bin/codex_overlay/` | `src/bin/voxterm/` | Rename (done) |
| Overlay docs | “Codex overlay” wording | “VoxTerm overlay” / “backend overlay” | Rename |
| UI labels | “Codex Output” (legacy TUI) | Keep (legacy-only); use backend label in overlay | Keep |
| Codex runtime types | `BackendEvent/BackendJob/BackendStats/BackendError` (`crate::codex`) | `CodexEvent/CodexJob/CodexJobStats/CodexBackendError` | Rename (done) |
| Codex runtime trait | `CodexBackend` (trait) | `CodexJobRunner` | Rename (done) |
| Codex CLI backend | `CliBackend` | `CodexCliBackend` | Rename (done) |
| IPC state field | `codex_backend` | `codex_cli_backend` | Rename (done) |
| Tests | `tests/codex_overlay_cli.rs` | `tests/voxterm_cli.rs` | Rename (done) |

**Status:** Done (rename map executed; Codex runtime names aligned; test rename applied).
| Legacy TUI module | `src/src/ui.rs` | `src/src/legacy_ui.rs` | Rename (done) |
| Legacy TUI app | `App` | `CodexApp` | Rename (done) |
| Legacy TUI entry | `run_app` | `run_legacy_ui` | Rename (done) |
| Legacy TUI dir | `src/src/app/` | `src/src/legacy_tui/` | Rename (done) |

### Phase 22: Overlay directory alignment
**Current:** `src/bin/voxterm/` (binary is `voxterm`).
**Plan options:**
- Keep `src/bin/voxterm/` and add module docs clarifying multi-backend purpose.
- Optional: rename to `src/bin/overlay/` if a more generic label is preferred.
**Status:** Renamed from `src/bin/codex_overlay/` to `src/bin/voxterm/` and updated docs/scripts.

### Phase 23: PTY session naming cleanup
**Candidates:**
- `PtyCodexSession` -> `PtyCliSession` (renamed).
- `spawn_codex_child` -> `spawn_pty_child` (renamed).
- Codex-specific comments in PTY docs -> generic CLI backend wording.

### Phase 24: App/UI naming cleanup
**Candidates:**
- Rename legacy TUI surface types to legacy names (`CodexApp`, `legacy_ui.rs`, `run_legacy_ui`).
- Rename the legacy TUI module directory to `legacy_tui/` to make the scope obvious.
- Keep codex-specific field names (`codex_job`, `codex_output`) inside the Codex-only TUI to avoid implying multi-backend support.
- If Codex TUI labels are ever reused elsewhere, switch to backend-labeled strings in that context.
- Keep actual Codex provider logic under `backend/codex.rs` and `codex/` modules; document the split to avoid confusion.

### Phase 25: Documentation + naming guidelines
- Add a short “Naming conventions” section to `dev/ARCHITECTURE.md`.
- Clarify the repo layout (`src/` workspace + crate at `src/src`) in `dev/DEVELOPMENT.md`.
**Status:** Done (naming conventions + layout note present; legacy_ui path updated).

**Verification**
- `cargo build --release --bin voxterm`
- `cargo test`

---

## Verification after phases

Minimum per phase:
```bash
cd src && cargo build --release --bin voxterm
cd src && cargo test
```

Mutation testing (per track milestone, not every phase):
```bash
cd src && cargo mutants --timeout 300 -- <MODULE_OR_DIR>
python3 ../dev/scripts/check_mutation_score.py --threshold 0.80
```

---

## Missed Mutations to Address (current)

All 6 escaped mutations are in `main.rs` control flow:
1. Line 193: `==` -> `!=` in `main()`
2. Line 197: Negation operator `!` deleted
3. Lines 199, 1424, 1478: Logic operator mutations (`&&` <-> `||`)
4. Line 199: Match guard replacements

**Strategy:** Add focused unit tests for each extracted module that cover all branches and guard conditions before moving to the next track.

---

## Expected Outcomes

| Metric | Before | After |
| --- | --- | --- |
| `main.rs` lines | 3366 | 400-600 |
| Large files >400 lines | 11 | 0-2 |
| `#[allow(clippy::too_many_arguments)]` | 4 | 0 |
| Mutation score | 4.41% | >80% (core modules) |
