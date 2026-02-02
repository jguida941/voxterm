# Rust Modularization Plan (Completed)

Goal: break up the largest Rust source files into cohesive modules without changing behavior,
keeping tests and mutation coverage intact. This is a structural refactor only.
Status: completed on 2026-01-29 (all phases checked below).

## Contents

- [Scope (top offenders by LOC)](#scope-top-offenders-by-loc)
- [Principles](#principles)
- [Proposed Module Layout](#proposed-module-layout)
- [Phased Execution Plan](#phased-execution-plan)
- [Test/CI Expectations](#testci-expectations)
- [Risks and Mitigations](#risks-and-mitigations)
- [Checklist](#checklist)

## Scope (top offenders by LOC, pre-split)

- `rust_tui/src/pty_session.rs` (~2.6k)
- `rust_tui/src/ipc.rs` (~2.6k)
- `rust_tui/src/bin/codex_overlay/main.rs` (~2.5k)
- `rust_tui/src/codex.rs` (~1.9k)
- `rust_tui/src/audio.rs` (~1.7k)
- `rust_tui/src/config.rs` (~1.3k)
- `rust_tui/src/app.rs` (~1.1k)

## Principles

- No behavior changes.
- Keep public API surface stable (`pub`/`pub(crate)` re-exports from the original module).
- Move tests with their code to avoid accidental coverage loss.
- Preserve mutation-test hooks and counters.
- Prefer mechanical moves + minimal renames in the first pass.

## Proposed Module Layout

### 1) codex_overlay (largest, highest churn)
Create `rust_tui/src/bin/codex_overlay/` with:

- `main.rs` (current main loop; bin entrypoint)
- `config.rs` (OverlayConfig, VoiceSendMode)
- `input.rs` (InputEvent, input parser, input thread)
- `writer.rs` (WriterMessage, writer thread, status redraw)
- `prompt.rs` (PromptTracker, prompt regex logic, prompt logger)
- `voice_control.rs` (VoiceManager + voice helpers; avoid colliding with `rust_tui::voice`)
- `transcript.rs` (pending queue, merge/flush logic)
- `tests.rs` (tests currently at file end)

### 2) ipc
Create `rust_tui/src/ipc/`:

- `mod.rs` (public API: run_ipc_mode, types)
- `protocol.rs` (IPC request/response types, serde)
- `router.rs` (command dispatch, provider selection)
- `session.rs` (provider session loops, stdio event loop)
- `tests.rs`

### 3) pty_session
Create `rust_tui/src/pty_session/`:

- `mod.rs` (PtyOverlaySession public API)
- `pty.rs` (pty open/fork, fd handling)
- `io.rs` (read/write helpers, output buffering)
- `osc.rs` (OSC/DSR/DA handling)
- `counters.rs` (test/mutants counters + overrides)
- `tests.rs`

### 4) codex backend
Create `rust_tui/src/codex/`:

- `mod.rs` (public API, re-exports)
- `backend.rs` (BackendJob, BackendEvent, stats)
- `pty_backend.rs` (CliBackend/PTY logic)
- `cli.rs` (process invocation and output capture)
- `tests.rs`

### 5) audio
Create `rust_tui/src/audio/`:

- `mod.rs` (public API, re-exports)
- `vad.rs` (VadConfig, engine, smoothing)
- `capture.rs` (CaptureState/FrameAccumulator/metrics)
- `resample.rs` (resampler paths)
- `dispatch.rs` (FrameDispatcher)
- `tests.rs`

### 6) config + app
Smaller splits to improve readability:

- `rust_tui/src/config/`:
  - `mod.rs` (AppConfig, VoicePipelineConfig)
  - `defaults.rs` (constants + defaults)
  - `validation.rs` (validate())
  - `tests.rs`

- `rust_tui/src/app/`:
  - `mod.rs` (App public API)
  - `logging.rs` (init_logging, log_debug)
  - `state.rs` (App fields + helpers)
  - `tests.rs`

## Current Layout (post-split, 2026-01-29)

- `rust_tui/src/bin/codex_overlay/`
  - `main.rs` (entrypoint)
  - `config.rs`, `input.rs`, `writer.rs`, `prompt.rs`, `voice_control.rs`, `transcript.rs`
- `rust_tui/src/ipc/` (`mod.rs`, `protocol.rs`, `router.rs`, `session.rs`, `tests.rs`)
- `rust_tui/src/pty_session/` (`mod.rs`, `pty.rs`, `io.rs`, `osc.rs`, `counters.rs`, `tests.rs`)
- `rust_tui/src/codex/` (`mod.rs`, `backend.rs`, `pty_backend.rs`, `cli.rs`, `tests.rs`)
- `rust_tui/src/audio/` (`mod.rs`, `vad.rs`, `capture.rs`, `resample.rs`, `dispatch.rs`, `tests.rs`)
- `rust_tui/src/config/` (`mod.rs`, `defaults.rs`, `validation.rs`, `tests.rs`)
- `rust_tui/src/app/` (`mod.rs`, `logging.rs`, `state.rs`, `tests.rs`)

## Phased Execution Plan

1) **codex_overlay** split (highest UX impact, fastest payoff)
   - Create directory + move structs/functions.
   - Keep `main()` in `main.rs`.
   - Avoid naming a module `voice` to prevent import collisions.

2) **pty_session** and **ipc** split
   - Keep test hooks with their modules.
   - Run `cargo test --bin voxterm` after each split.

3) **codex** backend split
   - Keep event structs and stats in `backend.rs`.

4) **audio** split
   - Move VAD + capture pipeline into separate modules.
   - Ensure tests follow their code.

5) **config/app** split
   - Keep CLI surface and validation intact.

## Test/CI Expectations

- `cargo fmt` and `cargo clippy --bin voxterm` at each phase.
- `cargo test` (or at least `cargo test --bin voxterm` + core lib tests).
- Mutation testing still executed in CI (no local changes needed).

## Risks and Mitigations

- **Mutation coverage regression**: keep test hooks with their modules; avoid logic changes.
- **Visibility mistakes**: re-export in `mod.rs` and use `pub(crate)` to avoid breakage.
- **Circular deps**: keep module boundaries strict; use small helper modules to break cycles.

## Checklist

### Phase 1: codex_overlay
- [x] Move config types to `config.rs`
- [x] Move input parsing + thread to `input.rs` (tests moved with it)
- [x] Move writer/status logic to `writer.rs` (tests moved with it)
- [x] Move prompt detection/logging to `prompt.rs`
- [x] Move voice manager + helpers to `voice_control.rs`
- [x] Move transcript queue/flush helpers to `transcript.rs`
- [x] Ensure `main.rs` imports from new modules cleanly
- [x] `cargo fmt`
- [x] `cargo clippy --bin voxterm`
- [x] `cargo test --bin voxterm`

### Phase 2: pty_session
- [x] Split `pty_session` into `pty.rs`, `io.rs`, `osc.rs`, `counters.rs`, `tests.rs`
- [x] `cargo fmt`
- [x] `cargo clippy --bin voxterm`
- [x] `cargo test`

### Phase 3: ipc
- [x] Split `ipc` into `protocol.rs`, `router.rs`, `session.rs`, `tests.rs`
- [x] `cargo fmt`
- [x] `cargo clippy --bin voxterm`
- [x] `cargo test`

### Phase 4: codex backend
- [x] Split `codex` into `backend.rs`, `pty_backend.rs`, `cli.rs`, `tests.rs`
- [x] `cargo fmt`
- [x] `cargo clippy --bin voxterm`
- [x] `cargo test`

### Phase 5: audio
- [x] Split `audio` into `vad.rs`, `capture.rs`, `resample.rs`, `dispatch.rs`, `tests.rs`
- [x] `cargo fmt`
- [x] `cargo clippy --bin voxterm`
- [x] `cargo test`

### Phase 6: config + app
- [x] Split `config` into `defaults.rs`, `validation.rs`, `tests.rs`
- [x] Split `app` into `logging.rs`, `state.rs`, `tests.rs`
- [x] `cargo fmt`
- [x] `cargo clippy --bin voxterm`
- [x] `cargo test`
