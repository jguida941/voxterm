# Architecture Decision: PTY Readiness + Auth Flow Stabilization

**Date**: 2026-01-11
**Status**: APPROVED
**Author**: Codex (AI assistant)

Previous: [docs/architecture/2026-01-10/](../2026-01-10/)

## Goal
- Remove the 20-30s PTY stall by treating any PTY output as progress and failing fast when output is only control bytes.
- Provide an explicit /auth login path that uses /dev/tty so IPC output stays clean.
- Ensure Codex slash command output is delivered even if the worker signal channel disconnects.

## Decision
- Add a PTY readiness handshake based on initial output and disable PTY if no output appears within the healthcheck window.
- Treat any PTY output as progress and fall back quickly when output is non-printable and goes quiet.
- Add /auth as both an IPC command and wrapper command; emit auth_start/auth_end events.
- Pause raw mode during /auth so provider CLIs can read from /dev/tty, then resume after login.
- Reset the Codex PTY session after auth to avoid stale unauthenticated state.
- Deliver Finished output even if the signal channel disconnects.

## Alternatives Considered
1. Keep the existing PTY health check (process alive only) and extend timeouts.
   - Pros: Minimal change.
   - Cons: Preserves 20-30s stalls and provides no real readiness signal.
2. Drop PTY and always use non-interactive CLI.
   - Pros: Predictable and simpler IO.
   - Cons: Loses persistent session state and Codex slash command parity.
3. Move to an app-server backend now.
   - Pros: More robust streaming and IO control.
   - Cons: Larger scope, parity and auth still unproven.

## Tradeoffs
- The readiness handshake can disable PTY when startup output is delayed or suppressed; this is acceptable to keep latency predictable.
- /dev/tty auth is Unix-specific; non-Unix platforms get a clear error instead of a broken flow.
- Auth requires temporarily disabling raw mode, which pauses local key handling while the provider CLI runs.

## Risks
- Provider login assumptions (uses `<cmd> login`) may require configuration if CLIs change.
- If Codex emits only control sequences for long periods, PTY may still wait until the overall timeout.

## Benchmarks
- Not run in this session.
- Recommended follow-ups:
  - `scripts/measure_latency.sh`
  - `scripts/tests/integration_test.sh`

## Impact on SDLC
- Added PTY readiness handshake, /auth flow, and auth events.
- Added mutation testing workflow and updated integration tests.
- Updated daily notes, changelogs, and navigation indexes.

## Approval
- User approved Option 1 on 2026-01-11 ("do it i approve 1").
