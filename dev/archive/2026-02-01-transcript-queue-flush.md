# Completed Items - 2026-02-01 (Transcript Queue Flush)

## Reliability
- [x] MP-001 Improve transcript queue flush reliability by allowing idle-based sends when prompt detection stalls after output completion.
- [x] Track PTY output idle time separately from auto-voice activity to avoid delaying flushes.
- [x] Add a regression test for idle-based flushing and update user-facing docs/changelog.
