# CI/CD Workflow Dependencies & Prerequisites

**Date**: 2025-11-12
**Status**: 📋 **REFERENCE GUIDE**
**Purpose**: Document all tools, secrets, and dependencies required for CI/CD pipeline

---

## Dependency Matrix

| Workflow | Tools Required | Secrets Required | External Services | Estimated Time |
|----------|----------------|------------------|-------------------|----------------|
| `docs-check.yml` | bash, git | None | None | < 2 min |
| `quality-check.yml` | cargo, rustfmt, clippy | None | None | < 3 min |
| `tests.yml` | cargo | None | None | < 7 min |
| `security.yml` | cargo-audit, cargo-geiger | None | GitHub Security | < 5 min |
| `coverage.yml` | cargo-tarpaulin | `CODECOV_TOKEN` | codecov.io | < 15 min |
| `benchmarks.yml` | cargo, criterion | None | GitHub Pages | < 15 min |
| `mutation-testing.yml` | cargo-mutants | None | None | ~120 min |

---

## Tool Installation Guide

### 1. Local Development Tools (for testing)

#### Install `act` (GitHub Actions local runner)

**macOS**:
```bash
brew install act
```

**Linux**:
```bash
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash
```

**Verify**:
```bash
act --version
# Expected: act version 0.2.x or higher
```

**Configuration** (optional, `.actrc`):
```bash
# Create .actrc in project root
echo "-P ubuntu-latest=catthehacker/ubuntu:act-latest" > .actrc
echo "--container-architecture linux/amd64" >> .actrc
```

#### Install Rust Toolchain

**Required**: Rust stable (1.88+)

```bash
# Install rustup (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install stable toolchain
rustup toolchain install stable

# Install required components
rustup component add rustfmt clippy
```

**Verify**:
```bash
cargo --version  # Should be 1.88+
rustfmt --version
clippy-driver --version
```

#### Install CI-Specific Cargo Tools

**cargo-audit** (security audits):
```bash
cargo install cargo-audit --locked
```

**cargo-geiger** (unsafe code detection):
```bash
cargo install cargo-geiger --locked
```

**cargo-tarpaulin** (code coverage):
```bash
cargo install cargo-tarpaulin --locked
```

**cargo-mutants** (mutation testing):
```bash
cargo install cargo-mutants --locked
```

**Verification**:
```bash
cargo audit --version
cargo geiger --version
cargo tarpaulin --version
cargo mutants --version
```

**Note**: These tools are NOT required for development, only for running CI workflows locally.

---

## External Service Setup

### 1. codecov.io (Code Coverage Tracking)

**Purpose**: Track code coverage over time, visualize coverage changes in PRs

**Setup Steps**:

1. **Sign up**: Go to https://codecov.io
2. **Connect GitHub**: Authorize Codecov to access your repositories
3. **Enable Repository**: Find `codex_voice` in the list and enable it
4. **Get Token**: Copy the upload token from the repository settings

**Add Token to GitHub Secrets**:
```bash
gh secret set CODECOV_TOKEN
# Paste token when prompted
```

**Verify**:
```bash
gh secret list | grep CODECOV_TOKEN
# Expected: CODECOV_TOKEN (updated X time ago)
```

**Cost**: FREE for public repositories, $10/month for private repositories

**Alternative**: Use GitHub Actions built-in coverage (no external service)

---

### 2. GitHub Pages (Benchmark Tracking)

**Purpose**: Host benchmark comparison reports (optional)

**Setup Steps**:

1. **Enable GitHub Pages**:
   ```bash
   gh api repos/:owner/:repo \
     -X PATCH \
     -f has_pages=true
   ```

2. **Configure Source**:
   - Go to repository Settings → Pages
   - Source: Deploy from a branch
   - Branch: `gh-pages` (will be created automatically)

3. **Verify**:
   - Visit https://YOUR_USER.github.io/codex_voice/
   - Should show "404" initially (will populate after first benchmark run)

**Cost**: FREE for public repositories

**Alternative**: Store benchmark results as artifacts (no hosting)

---

## GitHub Actions Configuration

### 1. Enable Actions

**Check if enabled**:
```bash
gh repo view --json hasActionsEnabled
```

**Enable if needed**:
```bash
gh api repos/:owner/:repo \
  -X PATCH \
  -f has_actions=true
```

### 2. Configure Branch Protection

**After workflows are validated**, enable branch protection:

```bash
gh api repos/:owner/:repo/branches/main/protection \
  -X PUT \
  -H "Accept: application/vnd.github+json" \
  -f required_status_checks[strict]=true \
  -f required_status_checks[contexts][]=docs-check \
  -f required_status_checks[contexts][]=quality-check \
  -f required_status_checks[contexts][]=tests \
  -f required_status_checks[contexts][]=security \
  -f enforce_admins=true \
  -f required_pull_request_reviews[required_approving_review_count]=1
```

**Validation**:
```bash
gh api repos/:owner/:repo/branches/main/protection | jq
```

**Expected**: JSON showing enabled protection rules

---

## Workflow Dependencies

### 1. Documentation Check Dependencies

**File**: `.github/workflows/docs-check.yml`

**Tools**:
- `bash` (standard)
- `git` (standard)
- `date` (GNU coreutils)

**Environment Variables**:
- `DATE` - Computed: `$(date +%Y-%m-%d)`

**File Dependencies**:
- `docs/architecture/YYYY-MM-DD/ARCHITECTURE.md` (must exist)
- `docs/architecture/YYYY-MM-DD/CHANGELOG.md` (must exist)
- `CHANGELOG.md` (must be updated in PR)
- `PROJECT_OVERVIEW.md` (must contain current date)

**Common Failures**:
- Missing daily folder → Create folder with ARCHITECTURE.md + CHANGELOG.md
- CHANGELOG not updated → Add entry to root CHANGELOG.md
- Stray root docs → Move to dated folder or archive

---

### 2. Quality Check Dependencies

**File**: `.github/workflows/quality-check.yml`

**Tools**:
- `cargo fmt` (via `rustfmt` component)
- `cargo clippy` (via `clippy` component)

**Cargo Components**:
```bash
rustup component add rustfmt clippy
```

**Cached Directories**:
- `~/.cargo/registry` (dependency cache)
- `~/.cargo/git` (git dependency cache)
- `rust_tui/target` (build cache)

**Cache Key**: `${{ runner.os }}-cargo-clippy-${{ hashFiles('rust_tui/Cargo.lock') }}`

**Common Failures**:
- Formatting issues → Run `cargo fmt` locally
- Clippy warnings → Fix warnings or use `#[allow(clippy::lint_name)]`

---

### 3. Test Dependencies

**File**: `.github/workflows/tests.yml`

**Tools**:
- `cargo test` (standard)

**Test Types**:
- Unit tests: `cargo test --lib`
- Integration tests: `cargo test --test '*'`
- Doc tests: `cargo test --doc`
- No-default-features: `cargo test --no-default-features`

**Platforms**:
- Ubuntu (linux-gnu)
- macOS (darwin)
- Windows (future, not yet implemented)

**Common Failures**:
- Failing tests → Fix tests
- Flaky tests → Add retry logic or fix test
- Platform-specific failures → Use conditional compilation

---

### 4. Security Audit Dependencies

**File**: `.github/workflows/security.yml`

**Tools**:
- `cargo audit` (requires installation)
- `cargo geiger` (requires installation)
- `dependency-review-action` (GitHub native)

**Installation in CI**:
```yaml
- name: Install cargo-audit
  run: cargo install cargo-audit --locked

- name: Install cargo-geiger
  run: cargo install cargo-geiger --locked
```

**Common Failures**:
- Vulnerable dependencies → Update dependencies or add exception
- Increased unsafe usage → Document rationale or refactor

**Advisory Database**:
- Cached: `~/.cargo/advisory-db`
- Updated: Daily via `cargo audit fetch`

---

### 5. Coverage Dependencies

**File**: `.github/workflows/coverage.yml`

**Tools**:
- `cargo tarpaulin` (requires installation)

**Secrets**:
- `CODECOV_TOKEN` (required)

**Configuration**:
```yaml
- name: Install tarpaulin
  run: cargo install cargo-tarpaulin --locked

- name: Generate coverage
  run: cargo tarpaulin --out Xml --output-dir coverage --fail-under 80
```

**Output**:
- `rust_tui/coverage/cobertura.xml` (coverage report)

**Thresholds**:
- **Minimum**: 80% line coverage
- **Fail CI**: If coverage < 80%

**Common Failures**:
- Coverage below threshold → Add more tests
- Tarpaulin timeout → Increase `--timeout` parameter
- Codecov upload failure → Check `CODECOV_TOKEN` secret

---

### 6. Benchmark Dependencies

**File**: `.github/workflows/benchmarks.yml`

**Tools**:
- `cargo bench` (standard)
- `criterion` (crate dependency)

**Configuration**:
```toml
# rust_tui/Cargo.toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "audio_pipeline"
harness = false
```

**Benchmark Files**:
- `rust_tui/benches/*.rs` (must exist)

**Output**:
- `rust_tui/target/criterion/` (reports)

**Common Failures**:
- No benchmarks exist → Create benchmark files first
- Benchmark regression → Investigate performance issue
- Timeout → Reduce benchmark iterations

---

### 7. Mutation Testing Dependencies

**File**: `.github/workflows/mutation-testing.yml`

**Tools**:
- `cargo mutants` (requires installation)

**Configuration**:
```yaml
- name: Install cargo-mutants
  run: cargo install cargo-mutants --locked

- name: Run mutation tests
  run: cargo mutants --timeout 300 --output mutants.json
```

**Output**:
- `rust_tui/mutants.json` (mutation report)

**Thresholds**:
- **Minimum**: 70% mutation score (caught mutants / total mutants)

**Runtime**:
- **Expected**: 1-3 hours (depends on test suite size)
- **Timeout**: 5 minutes per mutant

**Common Failures**:
- Mutation score too low → Add more tests
- Timeout → Increase `--timeout` parameter
- OOM (out of memory) → Reduce parallelism or increase runner memory

---

## Caching Strategy

### 1. Cargo Build Cache

**What to Cache**:
```yaml
- name: Cache cargo
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      rust_tui/target
    key: ${{ runner.os }}-cargo-${{ hashFiles('rust_tui/Cargo.lock') }}
```

**Cache Key Strategy**:
- Include `runner.os` (different for Ubuntu/macOS)
- Include `Cargo.lock` hash (invalidate on dependency change)
- Different keys for different workflows (avoid conflicts)

**Cache Invalidation**:
- Automatic on `Cargo.lock` change
- Manual via Actions UI (Settings → Actions → Caches)

**Benefits**:
- 5-10x faster builds (after first run)
- Reduces GitHub Actions minutes usage

---

### 2. Advisory Database Cache (for cargo-audit)

**What to Cache**:
```yaml
- name: Cache advisory-db
  uses: actions/cache@v4
  with:
    path: ~/.cargo/advisory-db
    key: advisory-db-${{ github.run_id }}
    restore-keys: advisory-db-
```

**Cache Strategy**:
- Use `restore-keys` (always restore, even if stale)
- Update will happen via `cargo audit fetch`
- Saves ~30 seconds per run

---

## Testing Workflows Locally

### Using `act`

**Test Single Workflow**:
```bash
# Test docs check
act pull_request -W .github/workflows/docs-check.yml

# Test quality checks
act pull_request -W .github/workflows/quality-check.yml

# Test tests
act pull_request -W .github/workflows/tests.yml
```

**Test Multiple Workflows** (parallel):
```bash
act pull_request \
  -W .github/workflows/docs-check.yml \
  -W .github/workflows/quality-check.yml \
  -W .github/workflows/tests.yml
```

**Dry Run** (validate syntax only):
```bash
act pull_request --dry-run
```

**With Secrets**:
```bash
# Create .secrets file
echo "CODECOV_TOKEN=your_token_here" > .secrets

# Run with secrets
act pull_request -W .github/workflows/coverage.yml --secret-file .secrets
```

**Limitations of `act`**:
- May not perfectly match GitHub Actions environment
- Some actions may not work (especially GitHub-native actions)
- Timing will differ (local is usually slower)

**Recommendation**: Use `act` for quick validation, then test on GitHub for real.

---

## Troubleshooting Common Issues

### Issue 1: Workflow Not Running

**Symptoms**: Workflow doesn't appear in PR checks

**Possible Causes**:
1. Path filters don't match changed files
2. Workflow syntax error (YAML invalid)
3. Branch protection not configured

**Debug**:
```bash
# Check workflow syntax
act --list  # Should show workflow

# Check if file paths match filters
git diff origin/main --name-only | grep "rust_tui/"
```

---

### Issue 2: Cache Not Working

**Symptoms**: Build takes full time every run

**Possible Causes**:
1. Cache key changing every run
2. Cache size exceeding limit (10 GB)
3. Different runner OS

**Debug**:
```bash
# Check cache status
gh api repos/:owner/:repo/actions/caches

# Clear all caches (if corrupted)
gh api repos/:owner/:repo/actions/caches --jq '.actions_caches[].id' | \
  xargs -I {} gh api -X DELETE repos/:owner/:repo/actions/caches/{}
```

---

### Issue 3: Secret Not Found

**Symptoms**: `Error: CODECOV_TOKEN secret not found`

**Fix**:
```bash
# Verify secret exists
gh secret list

# Add secret if missing
gh secret set CODECOV_TOKEN
```

---

### Issue 4: Tool Installation Fails

**Symptoms**: `cargo install cargo-audit` times out

**Fix**:
```yaml
# Use cached binary instead
- name: Install cargo-audit (cached)
  uses: taiki-e/install-action@v2
  with:
    tool: cargo-audit
```

**Alternative**: Pre-build Docker image with all tools installed

---

## Cost Optimization

### 1. Reduce Unnecessary Runs

**Strategy**: Use path filters to skip workflows when irrelevant files change

```yaml
on:
  pull_request:
    paths:
      - 'rust_tui/**'
      - '.github/workflows/tests.yml'
    paths-ignore:
      - '**.md'
      - 'docs/**'
```

**Benefit**: Save 50-70% of CI minutes on docs-only PRs

---

### 2. Use Matrix Strategically

**Before** (wasteful):
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    rust: [stable, beta, nightly]
```
**9 jobs** × 5 min = 45 min

**After** (optimized):
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest]
    rust: [stable]
```
**2 jobs** × 5 min = 10 min

**Rationale**: Only test on target platforms, only use stable Rust

---

### 3. Self-Hosted Runners for Nightly

**Cost**: FREE (use own hardware)

**Setup**:
1. Get a spare machine (old laptop, Raspberry Pi, cloud VM)
2. Install GitHub Actions runner
3. Configure runner for repository
4. Update nightly workflows to use self-hosted runner

**Configuration**:
```yaml
jobs:
  mutation:
    runs-on: self-hosted  # Instead of ubuntu-latest
```

**Trade-off**: Maintenance burden vs. cost savings

---

## Maintenance Schedule

### Daily
- [ ] Check nightly workflow results (mutation testing)
- [ ] Review security advisories (GitHub Security tab)

### Weekly
- [ ] Review CI metrics (timing, failure rate)
- [ ] Update cargo tools: `cargo install-update -a`
- [ ] Clear old caches if storage exceeds limit

### Monthly
- [ ] Review codecov trends (is coverage improving?)
- [ ] Review benchmark trends (any performance regressions?)
- [ ] Update dependencies: `cargo update`
- [ ] Audit GitHub Actions minutes usage

### Quarterly
- [ ] Review CI/CD pipeline effectiveness (are we catching bugs?)
- [ ] Update workflow actions to latest versions
- [ ] Re-evaluate cost vs. self-hosted runners
- [ ] Update this documentation

---

## Reference Links

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust CI/CD Best Practices](https://doc.rust-lang.org/cargo/guide/continuous-integration.html)
- [act - Local GitHub Actions](https://github.com/nektos/act)
- [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
- [cargo-mutants](https://github.com/sourcefrog/cargo-mutants)
- [codecov.io Documentation](https://docs.codecov.com)

---

**Status**: This dependency guide is COMPLETE. Ready for use during implementation.

*Next: Execute Phase 0 of [cicd_implementation_plan.md](cicd_implementation_plan.md)*
