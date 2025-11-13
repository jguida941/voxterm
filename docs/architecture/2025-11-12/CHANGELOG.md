# Daily Changelog — 2025-11-12

## Added
- Repository-level `CHANGELOG.md` to satisfy traceability requirements.
- Root `PROJECT_OVERVIEW.md` describing mission, current focus, and links to daily architecture notes.
- Initial daily architecture folder (`docs/architecture/2025-11-12/`) with `ARCHITECTURE.md` + daily changelog scaffolding.
- Expanded `master_index.md` responsibilities (navigation, daily update checklist).
- Migrated legacy docs (`docs/architecture.md`, developer/testing guides, Claude audit) into the standardized directories (`docs/architecture/YYYY-MM-DD/`, `docs/references/`, `docs/audits/`).
- Corrected historical architecture entries to use 2025-11-11 as the baseline (no pre-2025 records).
- Refreshed `docs/guides/architecture_overview.md`, `master_doc.md`, `HOW_TO_TEST_AUDIO.md`, and annotated `plan.md` as legacy context to reflect the current Rust pipeline and verified commands.
- Added `README.md`, the “You Are Here” block in `PROJECT_OVERVIEW.md`, and the new `docs/references/` hierarchy (`quick_start.md`, `testing.md`, `python_legacy.md`, `milestones.md`, `troubleshooting.md`) while archiving superseded guides in `docs/archive/OBSOLETE_GUIDES_2025-11-12/`.
- Updated helper scripts (`test_performance.sh`, `test_voice.sh`, `simple_test.sh`, `final_test.sh`) to use `cargo run`, Ctrl+R instructions, and the shared log location.
- Folded the CI/CD documentation (`cicd_plan`, `cicd_implementation_plan`, `cicd_dependencies`) into a single `docs/references/cicd_plan.md`, archiving the redundant references under `docs/archive/OBSOLETE_REFERENCES_2025-11-12/` for historical traceability.
- Added appendices to `docs/references/cicd_plan.md` (phase-by-phase scripts, tooling matrix, rollback/cost/troubleshooting guides) so the consolidated file now contains all detail from the archived references.
- Created `docs/audits/latency_remediation_plan_2025-11-12.md`, refreshed `PROJECT_OVERVIEW.md` to put latency stabilization first, and linked the audit plan throughout navigation so upcoming CI gates can reference a single remediation source.
- Expanded the latency plan with Phase 2A/2B/3 naming, explicit backpressure + frame contracts, shutdown/fallback policy, structured tracing expectations, and CI perf gate requirements; PROJECT_OVERVIEW now calls out Phase 2B as the active workstream.
- Further enhanced the latency plan with concrete failure-mode hierarchy, VAD safety rails, resource budgets, observability schema, CI hooks, and an execution timeline so Phase 2B work has production-grade guardrails.
- Updated `agents.md` latency requirements to reference the Phase 2B plan explicitly so every agent/CI run adheres to the production-grade specification.
- Captured state-machine, config surface, deployment profile, and concurrency guardrails in the latency plan so implementation details stay unambiguous.
- Tightened `agents.md` governance with Scope/Non-goals, "Before You Start" guidance, condensed voice constraints pointing to the latency plan, explicit doc-update rules, and a prominent end-of-session checklist.
- Logged the readiness audit (`docs/audits/READINESS_AUDIT_2025-11-12.md`), summarized its blockers in today’s architecture note, and documented the Phase 2A design (Earshot VAD) plus the immediate task list (feature gating, CI perf jobs, module decomposition planning).


See `docs/audits/latency_remediation_plan_2025-11-12.md` for the complete latency specification.
