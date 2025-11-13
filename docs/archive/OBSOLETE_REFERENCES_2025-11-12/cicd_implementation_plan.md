# CI/CD Implementation Plan — Step-by-Step Execution Guide

**Date**: 2025-11-12
**Status**: 📋 **READY FOR EXECUTION**
**Prerequisite**: Read [cicd_plan.md](cicd_plan.md) for technical architecture
**Aligned with**: agents.md lines 26-30, 164, 191

---

## Executive Summary

This document provides the **execution roadmap** for implementing the comprehensive CI/CD pipeline defined in `cicd_plan.md`. It breaks down the work into 4 phases with clear validation gates, dependency mapping, and rollback procedures.

**Timeline**: 4 weeks
**Effort**: ~20 hours total
**Risk Level**: LOW (incremental rollout with testing at each phase)

---

## Current State Analysis

### Existing CI/CD (`.github/workflows/rust_tui.yml`)

**What Works**:
- ✅ Runs on PR and push to `rust_tui/**`
- ✅ Has `cargo fmt --check`
- ✅ Has `cargo clippy -- -D warnings`
- ✅ Has `cargo test`
- ✅ Proper caching configured

**Gaps vs. agents.md Requirements**:
- ❌ No documentation enforcement (agents.md line 24, 191)
- ❌ No mutation testing (agents.md line 29)
- ❌ No benchmarks (agents.md line 25)
- ❌ No security audits (industry standard)
- ❌ No code coverage tracking (industry standard)
- ❌ Single platform only (no macOS/Windows)
- ❌ No post-merge comprehensive checks

**Compliance Score**: 42% (3/7 requirements met)

---

## Implementation Phases

### Phase 0: Pre-Implementation Setup (Week 0, 1 hour)

**Goal**: Prepare infrastructure before creating workflows

#### Tasks

1. **Verify GitHub repository settings**
   ```bash
   # Check if Actions are enabled
   gh repo view --json hasIssuesEnabled,hasWikiEnabled
   ```

2. **Install local testing tool**
   ```bash
   # Install 'act' for local workflow testing
   brew install act  # macOS
   # OR
   curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash
   ```

3. **Create secrets documentation**
   - Document required secrets: `CODECOV_TOKEN`
   - Plan for adding secrets after Phase 2 completion

4. **Backup existing workflow**
   ```bash
   cp .github/workflows/rust_tui.yml .github/workflows/rust_tui.yml.backup
   ```

5. **Create branch for CI work**
   ```bash
   git checkout -b ci/comprehensive-pipeline
   ```

**Validation Gate**:
- [ ] `act` tool installed and working: `act --list`
- [ ] Backup created: `ls -la .github/workflows/*.backup`
- [ ] CI branch created: `git branch --show-current`

**Rollback**: Delete branch, restore from backup

---

### Phase 1: Documentation Enforcement (Week 1, 3 hours)

**Goal**: Implement agents.md SDLC requirements (lines 24, 191)

**Priority**: **P0** (blocks PR merge)

#### Step 1.1: Create Documentation Check Workflow

**File**: `.github/workflows/docs-check.yml`

**Implementation**:
```bash
# Create new workflow file
touch .github/workflows/docs-check.yml
```

**Content**: Copy from `cicd_plan.md` lines 108-171 (Documentation Enforcement section)

**Key Features**:
- Checks for `docs/architecture/YYYY-MM-DD/ARCHITECTURE.md`
- Checks for `docs/architecture/YYYY-MM-DD/CHANGELOG.md`
- Verifies root `CHANGELOG.md` updated
- Guards against stray root architecture docs
- Warns if `PROJECT_OVERVIEW.md` "You Are Here" not updated

#### Step 1.2: Test Locally

```bash
# Test the docs-check workflow
act pull_request -W .github/workflows/docs-check.yml --dry-run

# If dry-run passes, test for real
act pull_request -W .github/workflows/docs-check.yml
```

**Expected Output**:
- If today's folder missing: **FAIL** with helpful message
- If CHANGELOG not updated: **FAIL** with agents.md reference
- If stray root docs exist: **FAIL**
- Otherwise: **PASS**

#### Step 1.3: Test Edge Cases

```bash
# Test 1: Missing daily folder (should FAIL)
rm -rf docs/architecture/$(date +%Y-%m-%d)
act pull_request -W .github/workflows/docs-check.yml

# Test 2: CHANGELOG not updated (should FAIL)
git checkout -- CHANGELOG.md  # Undo local changes
act pull_request -W .github/workflows/docs-check.yml

# Test 3: Stray root doc (should FAIL)
touch ARCHITECTURE.md
act pull_request -W .github/workflows/docs-check.yml
rm ARCHITECTURE.md

# Test 4: All good (should PASS)
# Restore proper state
git checkout -- .
act pull_request -W .github/workflows/docs-check.yml
```

#### Step 1.4: Commit and Test on GitHub

```bash
git add .github/workflows/docs-check.yml
git commit -m "Add documentation enforcement workflow

- Checks for daily architecture folder (agents.md line 24)
- Verifies CHANGELOG updated (agents.md line 23)
- Guards against root architecture docs (agents.md line 197)

Part of comprehensive CI/CD implementation (Phase 1)."

git push origin ci/comprehensive-pipeline
```

**Create test PR**:
```bash
gh pr create --title "CI Phase 1: Documentation Enforcement" \
  --body "Implements agents.md requirements for SDLC traceability.

**What This Adds**:
- ✅ Daily architecture folder check
- ✅ CHANGELOG enforcement
- ✅ Root docs guard

**Testing**:
- Tested locally with \`act\`
- Validated all edge cases (missing folder, no CHANGELOG, stray docs)

**References**:
- agents.md line 24 (daily architecture)
- agents.md line 191 (CI enforcement)
- docs/references/cicd_plan.md (full architecture)"
```

**Validation Gate**:
- [ ] Workflow appears in PR checks
- [ ] Workflow completes in < 2 minutes
- [ ] Workflow fails when docs missing (test by removing daily folder temporarily)
- [ ] Workflow passes when docs present

**Rollback**: Revert commit, delete workflow file

---

### Phase 2: Quality Checks & Tests (Week 1-2, 4 hours)

**Goal**: Consolidate and enhance existing checks

**Priority**: **P0** (blocks PR merge)

#### Step 2.1: Refactor Existing Workflow

**Current Issue**: `rust_tui.yml` does everything in one job (no parallelization)

**Improvement**: Split into separate workflows for faster feedback

**Create**: `.github/workflows/quality-check.yml`

**Implementation**:
```bash
touch .github/workflows/quality-check.yml
```

**Content**: Copy from `cicd_plan.md` lines 177-228 (Formatting & Linting section)

**Changes from existing**:
- Adds pedantic/nursery lints (stricter than current `-D warnings`)
- Separates formatting (1 min) from linting (2 min) for parallel execution
- Better caching configuration

#### Step 2.2: Enhance Test Workflow

**Create**: `.github/workflows/tests.yml`

**Implementation**:
```bash
touch .github/workflows/tests.yml
```

**Content**: Copy from `cicd_plan.md` lines 236-292 (Test Suite section)

**Enhancements over existing**:
- ✅ Multi-platform matrix (Ubuntu + macOS)
- ✅ Separate unit/integration/doc test stages
- ✅ Test with `--no-default-features` (catch feature flag issues)
- ✅ Better timeout configuration

#### Step 2.3: Deprecate Old Workflow

**Options**:
1. **Option A** (Recommended): Keep `rust_tui.yml` as legacy during transition
2. **Option B**: Delete immediately after new workflows proven

**Recommendation**: Option A for safety

**Action**: Add comment to `rust_tui.yml`:
```yaml
# DEPRECATED: This workflow will be removed after new pipeline is validated
# See: .github/workflows/quality-check.yml and .github/workflows/tests.yml
name: Rust TUI CI (LEGACY)
```

#### Step 2.4: Test New Workflows

```bash
# Test quality checks
act pull_request -W .github/workflows/quality-check.yml

# Test test suite
act pull_request -W .github/workflows/tests.yml

# Test both in parallel (simulates real PR)
act pull_request -W .github/workflows/quality-check.yml -W .github/workflows/tests.yml
```

**Expected Timing**:
- Quality checks: ~2 min
- Tests: ~3-5 min
- **Total PR feedback: < 7 minutes** (within 10 min target)

#### Step 2.5: Commit and Validate

```bash
git add .github/workflows/quality-check.yml
git add .github/workflows/tests.yml
git add .github/workflows/rust_tui.yml  # With DEPRECATED comment

git commit -m "Refactor CI into modular workflows

- Split fmt/clippy into quality-check.yml (2 min)
- Enhanced tests.yml with multi-platform matrix (5 min)
- Added no-default-features test (catch feature flag issues)
- Deprecated rust_tui.yml (will remove after validation)

Improves parallelization and feedback time.

Part of comprehensive CI/CD implementation (Phase 2)."

git push origin ci/comprehensive-pipeline
```

**Validation Gate**:
- [ ] All 3 workflows run in parallel
- [ ] Quality checks complete in < 2 min
- [ ] Tests complete in < 7 min
- [ ] Tests run on both Ubuntu and macOS
- [ ] No flaky tests (run 3 times to confirm)

**Rollback**: Remove new workflows, uncomment rust_tui.yml

---

### Phase 3: Security & Coverage (Week 2-3, 6 hours)

**Goal**: Add security audits and code coverage tracking

**Priority**: **P1** (important but not blocking)

#### Step 3.1: Security Audits

**Create**: `.github/workflows/security.yml`

**Implementation**:
```bash
touch .github/workflows/security.yml
```

**Content**: Copy from `cicd_plan.md` lines 300-356 (Security Audits section)

**Features**:
- `cargo audit` for dependency vulnerabilities
- `cargo geiger` for unsafe code tracking
- GitHub `dependency-review-action` for PR dependency changes
- Runs on schedule (daily at midnight) + PR

**Install Required Tools Locally** (for testing):
```bash
cargo install cargo-audit
cargo install cargo-geiger
```

**Test Locally**:
```bash
# Test security audit
cd rust_tui
cargo audit

# Test unsafe code check
cargo geiger

# Expected: Both should pass (or show current state)
```

**Decision Point**: Should security checks **block** PR merge?

**Recommendation**:
- `cargo audit`: **YES, block** (no vulnerable dependencies)
- `cargo geiger`: **NO, report only** (informational)

**Rationale**: We can't eliminate all `unsafe`, but we must eliminate known vulnerabilities.

#### Step 3.2: Code Coverage

**Create**: `.github/workflows/coverage.yml`

**Implementation**:
```bash
touch .github/workflows/coverage.yml
```

**Content**: Copy from `cicd_plan.md` lines 364-413 (Code Coverage section)

**Features**:
- Runs **post-merge only** (don't block PR)
- Uses `cargo tarpaulin` for coverage
- Uploads to codecov.io
- **Fails if coverage < 80%**

**Prerequisite**: Sign up for codecov.io

**Steps**:
1. Go to https://codecov.io
2. Connect GitHub account
3. Enable `codex_voice` repository
4. Get `CODECOV_TOKEN`

**Add Secret**:
```bash
gh secret set CODECOV_TOKEN
# Paste token when prompted
```

**Test Coverage Locally** (slow, ~10 min):
```bash
cargo install cargo-tarpaulin
cd rust_tui
cargo tarpaulin --out Xml --output-dir coverage
```

**Expected**: Coverage report generated in `rust_tui/coverage/`

**Question**: What is current coverage?

**Action**: Run locally and document baseline in `docs/architecture/2025-11-12/ARCHITECTURE.md`

#### Step 3.3: Commit Security + Coverage

```bash
git add .github/workflows/security.yml
git add .github/workflows/coverage.yml

git commit -m "Add security audits and code coverage

Security:
- cargo audit (blocks PR on vulnerabilities)
- cargo geiger (reports unsafe code usage)
- dependency-review-action (GitHub native)

Coverage:
- cargo tarpaulin (80% threshold)
- codecov.io integration
- Runs post-merge (don't block PR)

Part of comprehensive CI/CD implementation (Phase 3)."

git push origin ci/comprehensive-pipeline
```

**Validation Gate**:
- [ ] Security workflow runs on PR
- [ ] `cargo audit` passes (or fails with known issues documented)
- [ ] `cargo geiger` report appears in PR summary
- [ ] Coverage workflow runs post-merge
- [ ] Coverage report uploaded to codecov.io
- [ ] Coverage badge available

**Rollback**: Remove workflows, delete codecov integration

---

### Phase 4: Benchmarks & Mutation Testing (Week 3-4, 6 hours)

**Goal**: Performance tracking and test quality validation

**Priority**: **P2** (required by agents.md but not urgent)

#### Step 4.1: Benchmarks

**Create**: `.github/workflows/benchmarks.yml`

**Implementation**:
```bash
touch .github/workflows/benchmarks.yml
```

**Content**: Copy from `cicd_plan.md` lines 419-468 (Performance Benchmarks section)

**Features**:
- Runs **post-merge only** (expensive)
- Uses `cargo bench` with criterion
- Stores results in GitHub Pages
- Alerts on regressions > 10%

**Prerequisite**: Add benchmark to code

**Current State**: Check if benchmarks exist
```bash
ls -la rust_tui/benches/
```

**If no benchmarks exist**:
1. Create `rust_tui/benches/audio_pipeline.rs`
2. Add criterion benchmarks for:
   - Audio capture latency
   - Whisper transcription time
   - PTY session startup time

**Example Benchmark** (agents.md requires latency benchmarks):
```rust
// rust_tui/benches/audio_pipeline.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_audio_capture(c: &mut Criterion) {
    c.bench_function("audio_capture_1sec", |b| {
        b.iter(|| {
            // Benchmark 1 second of audio capture
            black_box(capture_audio_for_duration(1.0))
        })
    });
}

criterion_group!(benches, bench_audio_capture);
criterion_main!(benches);
```

**Test Benchmarks Locally**:
```bash
cd rust_tui
cargo bench
```

**Decision**: Implement benchmarks now or later?

**Recommendation**: Later (separate PR) unless latency work has started

**For now**: Create workflow file, mark as `continue-on-error: true` until benchmarks exist

#### Step 4.2: Mutation Testing

**Create**: `.github/workflows/mutation-testing.yml`

**Implementation**:
```bash
touch .github/workflows/mutation-testing.yml
```

**Content**: Copy from `cicd_plan.md` lines 476-523 (Mutation Testing section)

**Features**:
- Runs **nightly only** (very slow, ~2 hours)
- Uses `cargo mutants`
- **Requires 70% mutation score**
- Manual trigger available via `workflow_dispatch`

**Install Locally** (for testing):
```bash
cargo install cargo-mutants
```

**Test Mutation Testing Locally** (WARNING: SLOW):
```bash
cd rust_tui
cargo mutants --timeout 60 --in-place
```

**Expected**: Report showing how many mutants were caught by tests

**Current Baseline**: Document mutation score in daily architecture notes

**Decision**: Enable mutation testing now or wait until coverage > 80%?

**Recommendation**: Enable now but mark as `continue-on-error: true` until we improve tests

#### Step 4.3: Commit Benchmarks + Mutation Testing

```bash
git add .github/workflows/benchmarks.yml
git add .github/workflows/mutation-testing.yml

git commit -m "Add benchmarks and mutation testing

Benchmarks:
- cargo bench with criterion (agents.md line 25)
- Tracks performance over time
- Alerts on regressions > 10%
- Runs post-merge (expensive)

Mutation Testing:
- cargo mutants (agents.md line 29)
- 70% mutation score threshold
- Runs nightly (2 hours)
- Manual trigger available

Both marked continue-on-error until baselines established.

Part of comprehensive CI/CD implementation (Phase 4)."

git push origin ci/comprehensive-pipeline
```

**Validation Gate**:
- [ ] Benchmark workflow exists (even if no benches yet)
- [ ] Mutation workflow runs nightly
- [ ] Mutation report generated and uploaded as artifact
- [ ] Both workflows documented in README

**Rollback**: Remove workflows

---

## Post-Implementation Tasks

### Task 1: Enable Branch Protection

**After all workflows validated**, enable branch protection:

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
- [ ] Cannot merge PR without passing checks
- [ ] Cannot push directly to main
- [ ] Require 1 approval

### Task 2: Add CI Badges to README

Add to [README.md](../../README.md):

```markdown
## CI/CD Status

[![Documentation Check](https://github.com/YOUR_USER/codex_voice/actions/workflows/docs-check.yml/badge.svg)](https://github.com/YOUR_USER/codex_voice/actions/workflows/docs-check.yml)
[![Code Quality](https://github.com/YOUR_USER/codex_voice/actions/workflows/quality-check.yml/badge.svg)](https://github.com/YOUR_USER/codex_voice/actions/workflows/quality-check.yml)
[![Tests](https://github.com/YOUR_USER/codex_voice/actions/workflows/tests.yml/badge.svg)](https://github.com/YOUR_USER/codex_voice/actions/workflows/tests.yml)
[![Security Audit](https://github.com/YOUR_USER/codex_voice/actions/workflows/security.yml/badge.svg)](https://github.com/YOUR_USER/codex_voice/actions/workflows/security.yml)
[![codecov](https://codecov.io/gh/YOUR_USER/codex_voice/branch/main/graph/badge.svg)](https://codecov.io/gh/YOUR_USER/codex_voice)
```

### Task 3: Delete Legacy Workflow

After 2 weeks of new pipeline stability:

```bash
git rm .github/workflows/rust_tui.yml
git commit -m "Remove legacy CI workflow

New modular pipeline proven stable:
- docs-check.yml (< 2 min)
- quality-check.yml (< 2 min)
- tests.yml (< 5 min)
- security.yml (< 3 min)
- coverage.yml (post-merge, < 10 min)
- benchmarks.yml (post-merge, < 10 min)
- mutation-testing.yml (nightly, 2 hours)

Legacy rust_tui.yml no longer needed."
```

### Task 4: Update Documentation

Add section to [master_index.md](../../master_index.md):

```markdown
## CI/CD Pipeline

- **Workflows**: `.github/workflows/` (7 automated workflows)
- **Plan**: [`docs/references/cicd_plan.md`](docs/references/cicd_plan.md) (architecture)
- **Implementation Guide**: [`docs/references/cicd_implementation_plan.md`](docs/references/cicd_implementation_plan.md) (this document)

**Pipeline Stages**:
1. **PR Checks** (< 10 min, blocks merge): Docs, quality, tests, security
2. **Post-Merge** (< 20 min): Coverage, benchmarks
3. **Nightly** (2-3 hours): Mutation testing, fuzzing
```

### Task 5: Document in Daily Architecture Notes

Add to `docs/architecture/2025-11-12/ARCHITECTURE.md` or next day's folder:

```markdown
## CI/CD Implementation (2025-11-12)

Implemented comprehensive CI/CD pipeline per agents.md requirements:

**Implemented**:
- ✅ Documentation enforcement (agents.md line 24, 191)
- ✅ Quality checks (fmt, clippy with pedantic lints)
- ✅ Multi-platform tests (Ubuntu, macOS)
- ✅ Security audits (cargo audit, geiger, dependency-review)
- ✅ Code coverage tracking (tarpaulin, 80% threshold)
- ✅ Benchmarks (cargo bench, agents.md line 25)
- ✅ Mutation testing (cargo mutants, agents.md line 29)

**Baselines**:
- Code coverage: XX% (as of 2025-11-12)
- Mutation score: XX% (as of 2025-11-12)
- PR check time: ~7 minutes
- Post-merge time: ~20 minutes

**Next Steps**:
- Improve coverage to 80%
- Add specific benchmarks for audio pipeline latency
- Enable fuzzing for audio/UTF-8 modules
```

---

## Validation Checklist

**Phase 0: Pre-Implementation**
- [ ] `act` tool installed
- [ ] Existing workflow backed up
- [ ] CI branch created

**Phase 1: Documentation Enforcement**
- [ ] `docs-check.yml` created
- [ ] Tested locally with `act`
- [ ] Tested all edge cases (missing folder, no CHANGELOG, stray docs)
- [ ] Workflow runs on GitHub in < 2 min
- [ ] Properly fails when docs missing
- [ ] Properly passes when docs present

**Phase 2: Quality & Tests**
- [ ] `quality-check.yml` created
- [ ] `tests.yml` created
- [ ] Legacy workflow deprecated (not deleted yet)
- [ ] Tests run on Ubuntu + macOS
- [ ] All workflows run in parallel
- [ ] Total PR check time < 10 min
- [ ] No flaky tests

**Phase 3: Security & Coverage**
- [ ] `security.yml` created
- [ ] `coverage.yml` created
- [ ] codecov.io configured
- [ ] `CODECOV_TOKEN` secret added
- [ ] Security checks pass (or known issues documented)
- [ ] Coverage baseline documented
- [ ] Coverage badge added to README

**Phase 4: Benchmarks & Mutation**
- [ ] `benchmarks.yml` created
- [ ] `mutation-testing.yml` created
- [ ] Baseline benchmarks documented
- [ ] Mutation score baseline documented
- [ ] Both workflows run successfully (even if continue-on-error)

**Post-Implementation**
- [ ] Branch protection enabled
- [ ] CI badges added to README
- [ ] Legacy workflow deleted (after 2 weeks)
- [ ] Documentation updated (master_index.md)
- [ ] Daily architecture notes updated
- [ ] Repository CHANGELOG updated

---

## Rollback Procedures

### Emergency Rollback (if CI completely broken)

```bash
# Restore legacy workflow
git checkout origin/main -- .github/workflows/rust_tui.yml

# Delete new workflows
rm .github/workflows/docs-check.yml
rm .github/workflows/quality-check.yml
rm .github/workflows/tests.yml
rm .github/workflows/security.yml
rm .github/workflows/coverage.yml
rm .github/workflows/benchmarks.yml
rm .github/workflows/mutation-testing.yml

# Commit rollback
git commit -m "ROLLBACK: Restore legacy CI workflow

New pipeline had critical issues. Restoring rust_tui.yml.
Will investigate and retry."

git push origin ci/comprehensive-pipeline --force
```

### Partial Rollback (if specific workflow broken)

```bash
# Example: Remove security workflow if causing issues
git rm .github/workflows/security.yml
git commit -m "Temporarily remove security.yml (investigating issues)"
git push
```

---

## Cost Analysis

### GitHub Actions Minutes

**Free Tier**: 2,000 min/month
**Paid Tier**: $8/month for 3,000 min, then $0.008/min

**Estimated Monthly Usage**:
- 20 PRs × 10 min = 200 min
- 10 merges × 20 min = 200 min
- 30 nightly runs × 120 min = 3,600 min
- **Total: ~4,000 min/month**

**Cost**: $8/month base + $8 for extra 1,000 min = **~$16/month**

**Cost Reduction Options**:
1. Self-hosted runner for nightly jobs (free)
2. Reduce nightly frequency (weekly instead of daily)
3. Optimize mutation testing (run on changed modules only)

**Recommendation**: Accept $16/month cost (worth it for quality assurance)

---

## Success Metrics

**Track over 4 weeks**:
1. **CI Reliability**: Flaky test rate (target: < 1%)
2. **Feedback Time**: PR check duration (target: < 10 min)
3. **Coverage Trend**: Code coverage % (target: 80%)
4. **Mutation Score**: Test quality (target: 70%)
5. **Security Issues**: Vulnerability count (target: 0)
6. **Performance**: Benchmark trend (target: no regressions)

**Weekly Review Questions**:
- Are CI checks catching real issues?
- Are tests flaky? Which ones?
- Is PR feedback fast enough?
- Are we hitting cost budget?

---

## Conclusion

**This plan provides**:
1. ✅ Clear phase-by-phase implementation
2. ✅ Validation gates at each step
3. ✅ Rollback procedures for safety
4. ✅ Cost analysis and mitigation
5. ✅ Success metrics for tracking
6. ✅ Integration with existing agents.md workflow

**Timeline**:
- Week 1: Phase 0-2 (setup + docs/quality/tests)
- Week 2-3: Phase 3 (security + coverage)
- Week 3-4: Phase 4 (benchmarks + mutation)
- Post-implementation: Monitoring and refinement

**Total Effort**: ~20 hours over 4 weeks

**Risk**: LOW (incremental, well-tested, rollback-ready)

**Recommendation**: START WITH PHASE 0-1 THIS WEEK

---

**Next Step**: Review this plan with user, get approval, then execute Phase 0.
