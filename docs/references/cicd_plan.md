# CI/CD Master Plan — Production-Grade Pipeline

**Status**: 📋 **PLANNED / READY FOR EXECUTION**  \
**Priority**: **P1** (mandated by `agents.md`)  \
**Owners**: Platform/Infrastructure (reviews by project lead)  \
**Last reviewed**: 2025-11-12

This single document replaces the older `cicd_implementation_plan.md` and `cicd_dependencies.md`. It defines the pipeline architecture, all workflow dependencies, and the phased rollout plan that keeps us compliant with `agents.md` (lines 23-31, 164, 191).

---

## Contents
1. [Purpose & SDLC Alignment](#purpose--sdlc-alignment)  
2. [Pipeline Overview](#pipeline-overview)  
3. [Stage Specifications](#stage-specifications)  
4. [Dependencies & Tooling](#dependencies--tooling)  
5. [External Services & GitHub Settings](#external-services--github-settings)  
6. [Implementation Roadmap](#implementation-roadmap)  
7. [Update Cadence & Ownership](#update-cadence--ownership)  
8. [References](#references)

---

## Purpose & SDLC Alignment
`agents.md` requires: design-first workflow, daily architecture notes, CHANGELOG updates, unit + regression + mutation tests, and a **fast CI/CD pipeline** that enforces all of the above. The goal of this plan is to:
- Define the *complete* CI/CD surface (fast checks, tests, security, coverage, benchmarks, mutation).
- Capture every dependency (tools, secrets, services) needed to run those workflows.
- Provide a step-by-step rollout plan so we ship CI in controlled phases without breaking existing velocity.
- Ensure master navigation docs (`master_index.md`, `PROJECT_OVERVIEW.md`) can point to a **single source of truth** for CI/CD.

---

## Pipeline Overview
We follow a layered workflow that favors fast feedback on pull requests, deeper validation after merge, and heavy checks overnight.

### Feedback Cadence
- **Stage 1 – Fast Checks** (< 2 min): docs enforcement, formatting, linting. Blocks PR.
- **Stage 2 – Tests** (< 5 min): unit, integration, doc tests. Blocks PR.
- **Stage 3 – Security** (< 3 min): dependency audit, unsafe scan, dependency review. Blocks PR.
- **Post-merge** (< 15 min): coverage, benchmarks, multi-OS builds, artifact publish.
- **Nightly** (unbounded): mutation testing, fuzzing, extended SAST/DAST, license review.

### Architecture Diagram
```
PR Opened
 └─▶ Stage 1: Docs + Lint
      └─▶ Stage 2: Tests
           └─▶ Stage 3: Security
                └─▶ ✅ Merge Allowed
                      └─▶ Post-Merge Checks
                            └─▶ Nightly Deep Checks
```

---

## Stage Specifications
### Stage 1 – Fast Checks (< 2 min)
**Workflow**: `.github/workflows/docs-check.yml` + `.github/workflows/quality-check.yml`
- Documentation enforcement
  - Verify `docs/architecture/$TODAY/{ARCHITECTURE.md,CHANGELOG.md}` exist.
  - Confirm root `CHANGELOG.md` mutated in the PR.
  - Guard against stray root architecture files.
  - Warn if `PROJECT_OVERVIEW.md` "You Are Here" lacks today’s date.
- Formatting & linting
  - `cargo fmt --all -- --check` (Rust workspace).
  - `cargo clippy --all-targets -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo`.

### Stage 2 – Tests (< 5 min)
**Workflow**: `.github/workflows/tests.yml`
- Matrix on `ubuntu-latest` + `macos-latest` (Rust stable).
- Runs `cargo test --lib`, `cargo test --test '*'`, `cargo test --doc`.
- Extra job for `cargo test --no-default-features` to catch optional builds.

### Stage 3 – Security (< 3 min)
**Workflow**: `.github/workflows/security.yml`
- `cargo audit --deny warnings`.
- `cargo geiger` report appended to job summary.
- GitHub dependency review action with `fail-on-severity: moderate`.

### Post-Merge – Comprehensive (< 15 min)
**Workflow**: `.github/workflows/coverage.yml`
- `cargo tarpaulin --fail-under 80` and upload to Codecov.
- `cargo bench` (criterion) with artifact upload and optional GitHub Pages publish.
- Matrix build for Linux/macOS/Windows to ensure cross-platform health.

### Nightly – Deep Checks (unbounded)
**Workflows**: `.github/workflows/mutation-testing.yml`, `.github/workflows/fuzzing.yml`
- `cargo mutants` (target 70% mutation score).
- `cargo fuzz` runs per critical modules (audio, PTY parsing, UTF-8 sanitizer).
- Full SAST/DAST + license scanner.

---

## Dependencies & Tooling
### Workflow Dependency Matrix
| Workflow | Tools | Secrets | External Services | Est. Duration |
|----------|-------|---------|-------------------|---------------|
| `docs-check.yml` | bash, git | – | – | < 2 min |
| `quality-check.yml` | cargo, rustfmt, clippy | – | – | < 3 min |
| `tests.yml` | cargo | – | – | < 7 min |
| `security.yml` | cargo-audit, cargo-geiger | – | GitHub Security | < 5 min |
| `coverage.yml` | cargo-tarpaulin | `CODECOV_TOKEN` | codecov.io | < 15 min |
| `benchmarks.yml` | cargo, criterion | – | GitHub Pages (optional) | < 15 min |
| `mutation-testing.yml` | cargo-mutants | – | – | ~120 min |

### Local Tool Installation
```bash
# GitHub Actions local runner
brew install act   # macOS
# or
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash  # Linux

# Rust toolchain + required components
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install stable
rustup component add rustfmt clippy

# CI-specific cargo tools
cargo install cargo-audit --locked
cargo install cargo-geiger --locked
cargo install cargo-tarpaulin --locked
cargo install cargo-mutants --locked
```
**Verification**:
```bash
act --version
cargo --version && rustfmt --version && clippy-driver --version
cargo audit --version && cargo geiger --version
cargo tarpaulin --version && cargo mutants --version
```

### Optional `.actrc`
```
-P ubuntu-latest=catthehacker/ubuntu:act-latest
--container-architecture linux/amd64
```

---

## External Services & GitHub Settings
### Codecov (Coverage)
1. Sign into https://codecov.io, enable `codex_voice`.
2. Copy repository token → store in GitHub secret `CODECOV_TOKEN`.
3. Verify with `gh secret list | grep CODECOV_TOKEN`.

### GitHub Pages (Benchmarks)
1. Enable Pages via repo Settings → Pages → Deploy from branch (`gh-pages`).
2. Benchmarks workflow publishes HTML comparison each merge.
3. Optional but recommended for latency visibility.

### GitHub Actions Configuration
- Ensure Actions enabled: `gh repo view --json hasActionsEnabled`.
- Branch protection (after workflows are stable): require `docs-check`, `quality-check`, `tests`, `security` contexts, enforce admins, at least 1 approving review.

---

## Implementation Roadmap
Four phases (~20 hours total) roll these workflows out safely.

### Phase 0 — Preflight (Week 0, ~1 hr)
- Install `act`, verify Actions enabled, back up `.github/workflows/rust_tui.yml`.
- Create feature branch `ci/comprehensive-pipeline`.
- Document required secrets (Codecov) and owners.

### Phase 1 — Documentation Enforcement (Week 1, ~3 hrs)
- Add `docs-check.yml` exactly as specified above.
- Test locally with `act pull_request -W .github/workflows/docs-check.yml` (include negative tests for missing folders, stale CHANGELOG, stray root docs).
- Land via PR “CI Phase 1: Documentation Enforcement.” Blocks merges until SDLC traceability rules pass.

### Phase 2 — Quality & Tests (Week 1–2, ~4 hrs)
- Split existing `.github/workflows/rust_tui.yml` into `quality-check.yml` and `tests.yml` with caching + OS matrix.
- Add `cargo test --no-default-features` job.
- Validate with `act` on macOS + Linux images, then enforce via branch protection.

### Phase 3 — Security & Coverage (Week 2–3, ~6 hrs)
- Add `security.yml` (audit + geiger + dependency review).
- Add `coverage.yml` (tarpaulin + Codecov upload) triggered on `push main`.
- Document Codecov onboarding in `docs/references/quick_start.md` once live.

### Phase 4 — Benchmarks, Mutation, Nightly (Week 3–4, ~6 hrs)
- Add `benchmarks.yml` (criterion, artifact upload, optional Pages publish).
- Add `mutation-testing.yml` (cargo-mutants; schedule nightly + manual dispatch).
- Add `fuzzing.yml` for UTF-8 sanitizer/audio parsers.
- After stability, enable nightly badge in `PROJECT_OVERVIEW.md` metrics section.

**Rollback Strategy**: Every workflow is introduced via separate PR; revert the specific YAML file if instability occurs.

---

## Update Cadence & Ownership
- **When referencing CI/CD**: link to this file from `master_index.md`, `PROJECT_OVERVIEW.md`, and daily architecture notes.
- **End-of-session checklist** (`agents.md`): verify today’s architecture folder + CHANGELOG + “You Are Here,” and note CI/CD progress.
- **Documentation**: any change to workflows must be reflected here under the relevant stage + phase. Mention the update in the current day’s `docs/architecture/YYYY-MM-DD/` notes and both changelogs (daily + root).
- **CI Ownership**: Platform/Infra maintains workflows; feature teams ensure tests exist for new modules before requesting pipeline changes.

---

## References
- `agents.md` — SDLC + CI enforcement requirements.
- `docs/audits/claude_audit_nov12.md` — Original findings that triggered this work.
- `docs/audits/guides_redundancy_audit_nov12.md` — Documentation cleanup rationale.
- `docs/architecture/2025-11-12/ARCHITECTURE.md` — Daily record describing the governance rollout.

*Next Action*: Execute **Phase 1** and record progress in the current daily architecture folder before merging.

---

## Appendix A – Phase Playbooks & Validation Scripts

This appendix retains the detailed execution notes from the legacy implementation guide so every workflow rollout is deterministic.

### Phase 0 – Preflight Checklist
- **Tasks**
  1. Verify Actions enabled: `gh repo view --json hasActionsEnabled`.
  2. Install `act` (`brew install act` or curl installer on Linux) and confirm with `act --list`.
  3. Install/verify Rust stable + components: `rustup toolchain install stable && rustup component add rustfmt clippy`.
  4. Back up current workflow: `cp .github/workflows/rust_tui.yml .github/workflows/rust_tui.yml.backup`.
  5. Create branch `git checkout -b ci/comprehensive-pipeline`.
- **Validation Gate**: `act --version` ≥0.2, branch name correct, backup file present.
- **Rollback**: Delete branch and restore workflow from `.backup` if any preflight step fails.

### Phase 1 – Documentation Enforcement
- Create `.github/workflows/docs-check.yml` (Stage 1 spec) and run `act pull_request -W .github/workflows/docs-check.yml --dry-run`.
- Negative tests: remove today’s folder, reset CHANGELOG, add stray `ARCHITECTURE.md` — ensure workflow fails each case; restore files afterward.
- Commit + PR: “CI Phase 1: Documentation Enforcement.”
- **Validation Gate**: Workflow <2 minutes, deterministic failures for each missing artifact.
- **Rollback**: Revert workflow or temporarily guard with `if: false`.

### Phase 2 – Quality Checks & Tests
- Split legacy workflow into `quality-check.yml` (fmt + clippy) and `tests.yml` (unit/integration/doc + `--no-default-features`).
- Configure caches on `~/.cargo/registry`, `~/.cargo/git`, `rust_tui/target` keyed by `hashFiles('rust_tui/Cargo.lock')`.
- Validate via `act` for ubuntu + macOS containers.
- **Validation Gate**: Runtime <5 min per workflow; caches show “Cache restored from key …”.
- **Rollback**: Restore single workflow from backup.

### Phase 3 – Security & Coverage
- Add `security.yml` with `cargo audit`, `cargo geiger`, `actions/dependency-review-action@v4`.
- Add `coverage.yml` with `cargo tarpaulin --out Xml --fail-under 80` + Codecov upload (`CODECOV_TOKEN`).
- Update `README.md` badges once Codecov baseline is live.
- **Validation Gate**: Security workflow passes; Codecov dashboard receives upload.
- **Rollback**: Disable workflows temporarily or revert commit if blocking hotfixes.

### Phase 4 – Benchmarks, Mutation, Nightly
- `benchmarks.yml`: run `cargo bench`, upload `target/criterion` artifact, optionally push report to Pages.
- `mutation-testing.yml`: `cargo mutants --baseline mutants.baseline` nightly + manual dispatch; enforce ≥70% score.
- `fuzzing.yml`: `cargo fuzz run <target> -- -max_total_time=3600` for UTF-8, audio parsing, PTY sanitizer modules.
- **Validation Gate**: Benchmark artifacts downloadable; mutation score reported.
- **Rollback**: Scope heavy workflows to nightly schedule until stable.

### Post-Implementation Tasks
- Enable branch protection requiring `docs-check`, `quality-check`, `tests`, `security` contexts; enforce admins & ≥1 approval.
- Add CI badges (fmt/lint, tests, security, coverage, benchmarks, mutation) plus Codecov badge to `README.md`.
- Delete legacy `.github/workflows/rust_tui.yml` once replacements verified.
- Document completion in daily architecture notes + both changelogs.

---

## Appendix B – Tooling & Dependency Reference

Detailed dependency data from the legacy `cicd_dependencies.md`.

### Workflow Dependency Matrix
| Workflow | Tools/Binaries | Secrets | External Services | Est. Duration |
|----------|----------------|---------|-------------------|---------------|
| docs-check.yml | bash, git | – | – | <2 min |
| quality-check.yml | cargo, rustfmt, clippy | – | – | <3 min |
| tests.yml | cargo | – | – | <7 min |
| security.yml | cargo-audit, cargo-geiger | – | GitHub Security | <5 min |
| coverage.yml | cargo-tarpaulin | CODECOV_TOKEN | Codecov | <15 min |
| benchmarks.yml | cargo, criterion | – | GitHub Pages (optional) | <15 min |
| mutation-testing.yml | cargo-mutants | – | – | ~120 min |
| fuzzing.yml | cargo-fuzz, clang | – | – | configurable |

### Installation & Verification Commands
```bash
# Local Actions runner
brew install act  # macOS
# or
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash  # Linux
act --version

# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install stable
rustup component add rustfmt clippy

# CI-specific cargo tools
cargo install cargo-audit --locked
cargo install cargo-geiger --locked
cargo install cargo-tarpaulin --locked
cargo install cargo-mutants --locked
cargo install cargo-fuzz --locked
```

### Caching Strategy
- Cache `~/.cargo/registry`, `~/.cargo/git`, `rust_tui/target` with key `${{ runner.os }}-cargo-${{ hashFiles('rust_tui/Cargo.lock') }}` plus restore-keys fallback.
- Cache `~/.cargo/advisory-db` for `cargo-audit`.
- Optional `.actrc`:
  ```
  -P ubuntu-latest=catthehacker/ubuntu:act-latest
  --container-architecture linux/amd64
  ```

### External Services & Settings
- **Codecov**: enable repo, set `CODECOV_TOKEN` via `gh secret set`, verify with `gh secret list | grep CODECOV_TOKEN`.
- **GitHub Pages**: enable `gh-pages` deployment to host benchmark diffs.
- **GitHub Actions**: ensure enabled, then configure branch protection to require all blocking checks.

### Cost Optimization
- Use `paths:` filters to skip CI on docs-only changes.
- Limit macOS/Windows matrix jobs to nightly; rely on ubuntu during PRs.
- Schedule mutation/fuzzing overnight or on self-hosted runner to manage minutes.

---

## Appendix C – Operations, Troubleshooting & Maintenance

### Rollback Procedures
- **Emergency (all CI broken)**
  1. Temporarily disable new workflows (rename files or set `if: false`).
  2. Restore `.github/workflows/rust_tui.yml.backup`.
  3. Document incident + remediation in current architecture notes and changelog.
- **Partial (single workflow flaky)**
  1. `git revert <workflow commit>`.
  2. Replicate failure locally with `act` before reenabling.
  3. Track status under Risks in the day’s architecture note.

### Troubleshooting
| Issue | Symptom | Resolution |
|-------|---------|------------|
| Workflow not running | Missing checks on PR | Ensure `on` paths include touched files; confirm workflow filename under `.github/workflows/` matches action reference. |
| Cache never hits | Logs show `Cache not found` | Use stable key + `restore-keys`; avoid hashing paths that change per run. |
| Secret missing | Codecov upload fails | Add via `gh secret set CODECOV_TOKEN`; rerun workflow. |
| Tool install fails | `cargo install` timeout | Pin versions with `--locked`; preinstall via custom runner image. |

### Maintenance Schedule
- **Daily**: Verify today’s architecture folder + CHANGELOG exist; scan CI dashboard for failures.
- **Weekly**: Refresh advisory DB cache, review benchmark deltas.
- **Monthly**: Update Rust toolchain + cargo utilities pinned in workflows; reassess Codecov threshold.
- **Quarterly**: Revisit mutation targets, prune unused workflows, review GitHub Actions spend.

### Cost & Success Metrics
- Track Actions minutes per workflow; target PR check total <10 minutes.
- Maintain ≥80% coverage, ≥70% mutation score, and 0 undocumented merges (enforced via docs check).
- Record benchmark regressions >5% in architecture notes with mitigation plan.

### Reference Links
- GitHub Actions docs, Codecov setup, cargo-* manuals (legacy copies remain under `docs/archive/OBSOLETE_REFERENCES_2025-11-12/` for posterity).
