# Contributing

Thanks for taking the time to contribute.

## Before you start

- For non-trivial changes, open or comment on an issue first so we can align on scope.
- Keep docs and UX tables/controls lists in sync with actual behavior.
- Update `docs/CHANGELOG.md` for user-facing changes.
- Run verification before shipping (see `docs/dev/SDLC.md` for the full checklist).

## Development setup

- Install prerequisites in `docs/INSTALL.md`.
- Build the overlay:
  ```bash
  cd rust_tui && cargo build --release --bin voxterm
  ```

## Code style

- Rust: `cargo fmt` and `cargo clippy --workspace --all-features -- -D warnings`.
- Keep changes focused; prefer small, reviewable commits.

## Tests

Run what matches your changes:

```bash
cd rust_tui && cargo test
```

For overlay-only changes:

```bash
cd rust_tui && cargo test --bin voxterm
```

Targeted checks mirrored in CI (run when relevant):

```bash
# Perf smoke (voice metrics)
cd rust_tui && cargo test --no-default-features app::tests::perf_smoke_emits_voice_metrics -- --nocapture

# Memory guard (thread cleanup)
cd rust_tui && cargo test --no-default-features app::tests::memory_guard_backend_threads_drop -- --nocapture

# Mutation testing (heavy; usually on demand)
cd rust_tui && cargo mutants --timeout 300 -o mutants.out
python3 ../scripts/check_mutation_score.py --path mutants.out/outcomes.json --threshold 0.80
```

## Pull requests

- Explain the problem, the approach, and any tradeoffs.
- Include test output or notes on what was run.
- If UI output or flags change, update screenshots and docs that mention them.

## Security

For security concerns, see `.github/SECURITY.md`.
