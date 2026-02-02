# Completed Items - 2026-02-01 (Terminal Restore Guard)

## Reliability
- [x] Add a shared terminal restore guard and panic hook to clean up raw mode and alternate screen on crashes.
- [x] Add a minimal crash log entry on panic (metadata only unless log content is enabled).
- [x] Add a `--doctor` diagnostics report for terminal/config/audio visibility.
- [x] Clear overlay panel regions when height changes to avoid resize artifacts.
