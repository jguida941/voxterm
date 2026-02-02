# Developer Scripts

Scripts for development, testing, and releases.

**Tip:** Use the Makefile for common tasks: `make help`

Common pre-push checks:
```bash
make ci       # Core CI (fmt + clippy + tests)
make prepush  # All push/PR checks (ci + perf smoke + memory guard)
```

## Scripts

| Script | Purpose | Usage |
|--------|---------|-------|
| `release.sh` | Create GitHub release tag | `./dev/scripts/release.sh 1.0.33` |
| `update-homebrew.sh` | Update Homebrew formula | `./dev/scripts/update-homebrew.sh 1.0.33` |
| `mutants.py` | Interactive mutation testing | `python3 dev/scripts/mutants.py` |
| `check_mutation_score.py` | Verify mutation score | Used by CI |

## release.sh

Creates a Git tag for a new release.

```bash
./dev/scripts/release.sh 1.0.33
```

**Prerequisites:**
- On `master` branch
- No uncommitted changes
- `src/Cargo.toml` version matches
- `dev/CHANGELOG.md` has entry

## update-homebrew.sh

Updates the Homebrew tap formula with new version and SHA256.

```bash
./dev/scripts/update-homebrew.sh 1.0.33
```

## mutants.py

Interactive mutation testing with module selection.

```bash
# Interactive mode
python3 dev/scripts/mutants.py

# Test all modules
python3 dev/scripts/mutants.py --all

# Test specific module
python3 dev/scripts/mutants.py --module audio

# List available modules
python3 dev/scripts/mutants.py --list

# Show last results
python3 dev/scripts/mutants.py --results-only
```

Or use the Makefile:
```bash
make mutants         # Interactive
make mutants-audio   # Audio module only
make mutants-results # Show results
```

## tests/

Test scripts for benchmarking and integration testing.

| Script | Purpose |
|--------|---------|
| `benchmark_voice.sh` | Voice pipeline performance |
| `measure_latency.sh` | End-to-end latency profiling |
| `integration_test.sh` | IPC protocol testing |

---

## Release Workflow

```bash
# 1. Update version in src/Cargo.toml
# 2. Update dev/CHANGELOG.md
# 3. Commit all changes
git add -A && git commit -m "Release v1.0.33"

# 4. Create tag and push
./dev/scripts/release.sh 1.0.33

# 5. Create GitHub release
gh release create v1.0.33 --title "v1.0.33" --notes "See CHANGELOG.md"

# 6. Update Homebrew
./dev/scripts/update-homebrew.sh 1.0.33
```

Or use: `make release V=1.0.33 && make homebrew V=1.0.33`
