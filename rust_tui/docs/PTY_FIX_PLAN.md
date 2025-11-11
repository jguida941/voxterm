# PTY Integration Status (November 2025)

Codex Voice TUI now ships with a working persistent PTY session implemented in `src/pty_session.rs`.
This file tracks what is already finished and what remains, replacing the older unrealised plan that
referenced `portable-pty`/Tokio.

## Completed

- ✅ Custom PTY session backed by `libc::openpty`, with automatic TERM exports and CSI replies.
- ✅ Persistent sessions are enabled by default via `App::ensure_codex_session`.
- ✅ PTY read loop now waits for quiet periods instead of bailing after the first 500 ms poll.
- ✅ All Codex fallbacks share the same newline-terminated prompt, so exec/pty both run reliably.

## In Progress / Future Ideas

| Area | Status | Notes |
| --- | --- | --- |
| Streaming UI updates | ⚠️ Pending | The event loop still disables background streaming to avoid cursor queries; re-enable once the sanitiser fully handles them. |
| Cross-platform backend | ⚠️ Optional | `libc::openpty` is Unix-only. Evaluate `portable-pty` if/when Windows support matters. |
| Async PTY tasks | ⚠️ Optional | Voice capture already runs off-thread, but PTY writes/reads remain synchronous; revisit after streaming UI work. |

## Testing Checklist

- [x] Prompt sent via persistent PTY matches one-shot CLI output.
- [x] CSI responses keep Codex from hanging on cursor/device queries.
- [ ] Background streaming re-enabled without corrupting the UI.
- [ ] Session recovery after Codex crash (`App::ensure_codex_session` restarts).

Update this document whenever PTY behavior changes so new contributors know the current state of play.
