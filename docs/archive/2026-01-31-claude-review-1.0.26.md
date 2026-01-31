# Release Review: 1.0.26 (2026-01-31)

## Summary
- Reviewed visual system integration (status line, help overlay, theme selection, mic meter output).
- Addressed status line refresh edge cases and help overlay sizing.
- Verified release build and full test suite locally.

## Reviewed areas
- Overlay status rendering and truncation behavior.
- Help overlay rendering and resize behavior.
- Theme selection and color mode fallback.
- Session stats output on exit.

## Verification
- `cd rust_tui && cargo fmt --all -- --check`
- `cd rust_tui && cargo clippy --workspace --all-features -- -D warnings`
- `cd rust_tui && cargo test`
- `cd rust_tui && cargo build --release --bin codex-voice`

## Notes / risks
- `?` opens the help overlay and cannot be sent through while the overlay is active (documented in user docs).
- Real-time audio level display remains a future enhancement (tracked in visual plan).

