# Daily Changelog - 2026-01-11

## Session Summary

Stabilized the PTY path to avoid 20-30s stalls, added /auth login via /dev/tty for Codex and Claude, and ensured Codex output is delivered even if the signal channel disconnects.

## Changes Made

### Rust Backend
- Added PTY readiness handshake and fast fallback when only control output appears.
- Added /auth command (IPC + wrapper) that runs provider login via /dev/tty.
- Added auth_start/auth_end IPC events and reset Codex PTY session after auth.
- Fixed Codex event draining on signal channel disconnect so Finished output is not lost.
- Added unit tests for /auth command parsing and event serialization.

### TypeScript CLI
- Added /auth command and help text.
- Added auth_start/auth_end handling with raw mode toggling during login.
- Refactored raw mode handling to enable/disable cleanly and avoid input conflicts.

### CI and Tests
- Added mutation-testing workflow for nightly/manual runs.
- Updated integration test script to include auth command JSON validity.

## Bugs Fixed

1. PTY stalls caused by control-only output now fail fast and disable PTY.
2. Codex output was dropped when the signal channel disconnected.
3. No supported login flow for Codex/Claude in IPC mode.

## Files Modified

- `rust_tui/src/pty_session.rs`
- `rust_tui/src/codex.rs`
- `rust_tui/src/ipc.rs`
- `ts_cli/src/index.ts`
- `ts_cli/src/bridge/rust-ipc.ts`
- `scripts/tests/integration_test.sh`
- `.github/workflows/mutation-testing.yml`
- `docs/architecture/2026-01-11/ARCHITECTURE.md`
- `docs/architecture/2026-01-11/CHANGELOG.md`
- `CHANGELOG.md`
- `PROJECT_OVERVIEW.md`
- `master_index.md`

## Status

COMPLETE - PTY readiness, auth flow, and output delivery improvements implemented.

## Test Results

- Not run in this session. Recommended: `cargo test --workspace --all-features` and `scripts/tests/integration_test.sh`.
