# Release Review: 1.0.27 (2026-01-31)

## Summary
- Updated launcher tables to surface help overlay and theme options.
- Documented overlay visual system in architecture docs.
- Added modularization audit plan doc for reference.

## Reviewed areas
- `start.sh` startup tables and theme/help messaging.
- Architecture doc alignment with overlay visuals.
- Release version bump for Cargo.toml and Info.plist.

## Verification
- `cd rust_tui && cargo build --release --bin codex-voice`

## Notes / risks
- Help overlay is triggered with `?` after the overlay starts (launcher screen is static).
