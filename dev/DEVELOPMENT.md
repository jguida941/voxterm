# Development

## Contents

- [Project structure](#project-structure)
- [Building](#building)
- [Testing](#testing)
- [Contribution workflow](#contribution-workflow)
- [Code style](#code-style)
- [Testing philosophy](#testing-philosophy)

## Project structure

```
voxterm/
├── .github/
│   ├── CONTRIBUTING.md   # Contribution guidelines
│   ├── SECURITY.md       # Security policy
│   └── workflows/        # CI workflows
├── app/
│   ├── macos/VoxTerm.app # macOS double-click launcher
│   └── windows/          # Windows launcher (planned)
├── agents.md             # SDLC policy and release checklist (local)
├── QUICK_START.md        # Fast setup and commands
├── guides/
│   ├── CLI_FLAGS.md        # Full CLI and env reference
│   ├── INSTALL.md          # Install options and PATH notes
│   ├── TROUBLESHOOTING.md  # Common issues and fixes
│   ├── USAGE.md            # Controls and overlay behavior
│   └── WHISPER.md          # Whisper model guidance
├── dev/
│   ├── ARCHITECTURE.md     # Architecture diagrams and data flow
│   ├── CHANGELOG.md        # Release history
│   ├── DEVELOPMENT.md      # Build/test workflow
│   ├── active/             # Active plans and work in progress
│   ├── archive/            # Completed work entries
│   ├── adr/                # Architecture decision records
│   └── scripts/            # Developer scripts
│       ├── release.sh         # GitHub release script
│       ├── update-homebrew.sh # Homebrew tap update
│       ├── check_mutation_score.py # Mutation score helper
│       └── tests/             # Test scripts
├── img/                 # Screenshots
├── src/                 # Rust overlay + voice pipeline
│   └── src/
│       ├── bin/codex_overlay/main.rs # Overlay entry point
│       ├── app/         # TUI state + logging
│       ├── audio/       # CPAL recording, VAD, resample
│       ├── backend/     # AI CLI backend presets
│       ├── codex/       # Provider backend + PTY worker
│       ├── config/      # CLI flags + validation
│       ├── ipc/         # JSON IPC mode
│       ├── pty_session/ # PTY wrapper
│       ├── voice.rs     # Voice capture orchestration
│       ├── mic_meter.rs # Ambient/speech level sampler
│       ├── stt.rs       # Whisper transcription
│       └── ui.rs        # Full TUI rendering
├── scripts/
│   ├── README.md        # Script documentation
│   ├── install.sh       # One-time installer
│   ├── start.sh         # Linux/macOS launcher
│   ├── setup.sh         # Model download and setup
│   └── python_fallback.py # Python fallback pipeline
├── whisper_models/      # Whisper GGML models
└── bin/                 # Install wrapper (created by install.sh)
```

## Building

```bash
# Rust overlay
cd src && cargo build --release --bin voxterm

# Rust backend (optional dev binary)
cd src && cargo build --release
```

## Testing

```bash
# Rust tests
cd src && cargo test

# Overlay tests
cd src && cargo test --bin voxterm

# Perf smoke (voice metrics)
cd src && cargo test --no-default-features app::tests::perf_smoke_emits_voice_metrics -- --nocapture

# Memory guard (thread cleanup)
cd src && cargo test --no-default-features app::tests::memory_guard_backend_threads_drop -- --nocapture

# Mutation tests (CI enforces 80% minimum score)
cd src && cargo mutants --timeout 300 -o mutants.out
python3 ../dev/scripts/check_mutation_score.py --path mutants.out/outcomes.json --threshold 0.80
```

## Contribution workflow

- Open or comment on an issue for non-trivial changes so scope and UX expectations are aligned.
- Keep UX tables/controls lists and docs in sync with behavior.
- Update `dev/CHANGELOG.md` for user-facing changes and note verification steps in PRs.

## Code style

- Rust: run `cargo fmt` and `cargo clippy --workspace --all-features -- -D warnings`.
- Keep changes small and reviewable; avoid unrelated refactors.
- Prefer explicit error handling in user-facing flows (status line + logs) so failures are observable.

## Testing philosophy

- Favor fast unit tests for parsing, queueing, and prompt detection logic.
- Add regression tests when fixing a reported bug.
- Run at least `cargo test` locally for most changes; add targeted bin tests for overlay-only work.

## CI/CD Workflow

GitHub Actions run on every push and PR:

| Workflow | File | What it checks |
|----------|------|----------------|
| Rust TUI CI | `.github/workflows/rust_ci.yml` | Build, test, clippy, fmt |
| Perf Smoke | `.github/workflows/perf_smoke.yml` | Perf smoke test + metrics verification |
| Memory Guard | `.github/workflows/memory_guard.yml` | 20x memory guard loop |
| Mutation Testing | `.github/workflows/mutation-testing.yml` | 80% minimum mutation score (scheduled) |

**Before pushing, run locally (recommended):**

```bash
# Core CI (matches rust_ci.yml)
make ci

# Full push/PR suite (adds perf smoke + memory guard loop)
make prepush
```

**Manual equivalents (if you prefer direct cargo commands):**

```bash
cd src

# Format code
cargo fmt

# Lint (must pass with no warnings)
cargo clippy --workspace --all-features -- -D warnings

# Run tests
cargo test --workspace --all-features

# Check mutation score (optional, CI enforces this)
cargo mutants --timeout 300 -o mutants.out
python3 ../dev/scripts/check_mutation_score.py --path mutants.out/outcomes.json --threshold 0.80
```

**Check CI status:** [GitHub Actions](https://github.com/jguida941/voxterm/actions)

## Releasing

### Version bump

1. Update version in `src/Cargo.toml`
2. Update `dev/CHANGELOG.md` with release notes
3. Commit: `git commit -m "Release vX.Y.Z"`

### Create GitHub release

```bash
# Use the release script (recommended)
./dev/scripts/release.sh X.Y.Z

# Create release on GitHub
gh release create vX.Y.Z --title "vX.Y.Z" --notes "See CHANGELOG.md"
```

### Update Homebrew tap

```bash
# Use the update script (recommended)
./dev/scripts/update-homebrew.sh X.Y.Z
```

This automatically fetches the SHA256 and updates the formula.

Users can then upgrade:
```bash
brew update && brew upgrade voxterm
```

See `scripts/README.md` for full script documentation.

## Local development tips

**AI review notes** (e.g., `claude_review.md`) are local-only, gitignored, and kept in the repo root for session notes.

**Test with different backends:**
```bash
voxterm              # Codex (default)
voxterm --claude     # Claude Code
voxterm --gemini     # Gemini CLI (in works; not yet supported)
```

**Debug logging:**
```bash
voxterm --logs                    # Enable debug log
tail -f $TMPDIR/voxterm_tui.log   # Watch log output
```
