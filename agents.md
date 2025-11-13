# AgentMD: Codex Voice-to-Codex Wrapper

This document defines how agents (Codex and others) must operate on this repository: SDLC rules, architecture constraints, latency policy, documentation standards, and daily workflow.

## Scope

- Applies to all automated agents and contributors working on the `codex_voice` project (Rust TUI wrapper, voice pipeline, CI, documentation automation).
- Covers SDLC discipline, architecture guardrails, latency requirements, documentation expectations, and workflow control.

## Non-goals

- Does not define detailed security hardening beyond the CI checks described here.
- Does not prescribe UX copy or visual design.
- Does not grant agents permission to restructure the project root or move major files/directories without explicit human approval.

## Before You Start

1. Read this file and any linked design docs relevant to the task (e.g., the latency plan).
2. Identify the exact sections that apply; confirm instructions are unambiguous.
3. Ask for approval if any requirement conflicts or is unclear.
4. Only after understanding the rules, begin editing or coding.

⸻

Codex Requirements Summary for Rust Voice-to-Codex Wrapper

1. SDLC Discipline (Mandatory)

Codex must follow a structured SDLC process. Every change must include:
	1.	Design reasoning before coding
	•	Explain the design choice
	•	Provide alternative options
	•	State why the selected design is optimal
	•	Ask for approval before implementing any nontrivial architectural change
	2.	Implementation with traceability
	•	All variables, methods, structs, enums must be self-describing
	•	No cryptic names (no a, x1, buf1 unless it represents a specific abstraction)
	•	Every module needs docstrings and concise comments
	•	Every change must update a CHANGELOG entry
	•	Architectural notes must be written daily under docs/architecture/YYYY-MM-DD/
	•	Document: what changed, why, alternatives considered, tradeoffs, benchmarks
	3.	Testing requirements
	•	Unit tests for every module
	•	Regression tests for major features
	•	Mutation testing when applicable
	•	Fast CI/CD pipeline must be set up early (GitHub Actions or equivalent)

⸻

2. Coding Requirements

General
	•	Code must be modular, not monolithic
	•	Target file size: 200–300 lines max
	•	Everything should fit together like composable components
	•	No hidden global state
	•	Explicit error handling using Result<T, E>
	•	Logging must be minimal, async, and configurable

Rust-Specific
	•	Use idiomatic Rust
	•	Use proper ownership and lifetime management
	•	Avoid unnecessary cloning
	•	Investigate potential race conditions in the async-runtime
	•	Audit all blocking calls inside async functions
	•	Never introduce Python fallbacks or Python-like architecture
	•	Performance must match what is expected from Rust (no unexplained latency)

Python Reference
	•	Python is only a fallback reference, not a runtime dependency
	•	Codex must document when Rust behavior differs from Python
	•	AgentMD must contain pointers to the Rust std docs and Python std docs

⸻

3. Voice-to-Codex Wrapper Requirements

Codex must audit the following:
- **Latency**
  - Round-trip from voice input to Codex output should not exceed a few hundred milliseconds; 10 seconds is unacceptable.
  - Investigate race conditions, async vs sync boundaries, blocking file/log I/O, misconfigured channels, cross-thread contention, and unintended Python fallbacks.
  - Provide diagnostic logs and a performance trace for each call path.
  - Voice pipeline must comply with `docs/audits/latency_remediation_plan_2025-11-12.md`, including: non-blocking audio callback, bounded SPSC queue (drop-oldest backpressure), enforced resource limits, state-machine lifecycle, graded fallback ladder (streaming → batch → dev-only Python → manual), and per-request latency metrics + CI SLAs.
	2.	Architecture Goal
	•	The wrapper must sit on top of Codex, not Codex on top of the wrapper
	•	The wrapper extends Codex with:
	•	Voice input
	•	Standardized formatting
	•	Module structure
	•	CI/CD and testing harness
	•	Future IDE-style tooling
	•	Codex must not overengineer internals or reinvent subsystems unnecessarily
	•	Codex must preserve the architecture unless explicit approval is granted

⸻

4. Interaction Rules for Codex

Codex must obey the following rules on every interaction:
	1.	No autonomous coding
	•	Do not modify architecture without prior approval
	•	For every proposed change, provide:
	•	Explanation
	•	Alternatives
	•	Tradeoffs
	•	Recommendation
	•	Wait for approval before producing code
	2.	Explain everything
	•	Every change must have:
	•	Reasoning
	•	Explanation of purpose
	•	Expected effect
	•	Impact on SDLC
	•	Possible risks
	3.	Fully traceable output
	•	Insert docstrings and comments
	•	Include architectural notes
	•	Update change logs
	•	Use clear variable and method names
	•	Map each module to a single responsibility
	4.	No over-engineering
	•	Prefer the simplest correct solution
	•	If complexity is required, state why
	•	If Codex proposes an advanced design, it must justify the value and ask before implementing

⸻

Final Instruction Block to Give Codex

Paste this block exactly:

⸻

INSTRUCTION TO CODEX

You are not allowed to code until you first:
	1.	Explain the problem
	2.	Identify race conditions or performance bottlenecks
	3.	Propose 2 to 4 architectural approaches
	4.	Ask for approval
	5.	After approval, produce:
	•	Modular Rust code
	•	Clean variable and method names
	•	Docstrings and comments
	•	Updated CHANGELOG
	•	Architecture notes under dated folder
	•	Unit tests, regression tests, mutation tests
	•	CI config
	•	Benchmarks for latency improvements

You must follow SDLC. No hidden complexity. No unapproved architecture. No Python fallbacks.
Your job is to audit the Rust code for performance, race conditions, async errors, blocking I/O, misconfigured channels, or anything that could cause multi-second delays.
Produce detailed reasoning before making changes and request confirmation before generating any code.

⸻

Reference documentation:

- Rust standard library: https://doc.rust-lang.org/std/
- Python standard library: https://docs.python.org/3/library/

⸻

## Architectural Integration Plan

1. Wrapper as an Extension Layer
	The Rust wrapper must remain a thin, modular extension that sits on top of Codex. It should intercept Codex interactions, add capabilities such as voice capture or additional slash commands, and avoid re-implementing Codex’s core features so the underlying architecture stays stable.

2. Modular Code and Documentation
	Each capability ships as a self-contained module (target 200–300 LOC) with docstrings. Every working day produces a dated `docs/architecture/YYYY-MM-DD/` note summarizing changes, rationale, alternatives, tradeoffs, and benchmarks, alongside a CHANGELOG entry for traceability.

3. Clear Naming and Commenting
	All identifiers must be self-describing. Reserve comments for non-obvious logic. No cryptic abbreviations or implicit behavior.

4. Pipeline and CI/CD Alignment
CI must run formatting, clippy, tests (unit + regression + mutation), and latency benchmarks. Modules land only with accompanying tests. Automation should fail fast when architectural notes or CHANGELOG updates are missing.

Any change to code must also:
- Update the current day’s `docs/architecture/YYYY-MM-DD/ARCHITECTURE.md` with design reasoning.
- Update the same day’s `docs/architecture/YYYY-MM-DD/CHANGELOG.md` with concrete changes.
- Ensure `master_index.md` points to the latest dated folder.

CI must reject any change that modifies code without these documentation updates.

5. Guided Scope Control for Codex
	Codex proposes 2–3 architectural approaches for any new feature, details pros/cons, and waits for approval before writing code. The wrapper may not over-engineer or diverge from Codex’s architecture without explicit authorization.

⸻

## Organizational Plan for Traceability

1. Main Project Overview File
	Maintain a single immutable `PROJECT_OVERVIEW.md` (or equivalent) at repo root with:
	- High-level goals and scope
	- List of major architectural decisions (each linking into the dated architecture folders)
	- A “Current daily notes” pointer linking to the most recent `docs/architecture/YYYY-MM-DD/` directory

2. Daily Architectural Folders
	For every working day create `docs/architecture/YYYY-MM-DD/` containing:
	- `ARCHITECTURE.md` summarizing the day’s design decisions, alternatives, tradeoffs, benchmarks
	- A daily `CHANGELOG.md` snippet documenting incremental changes
	- Any diagrams or supporting files needed for that day

3. Linking and Navigation
	Each daily `ARCHITECTURE.md` begins with a link to the previous day’s folder, forming a breadcrumb trail for reviewers.

4. CI/CD Enforcement
	Pull requests must fail unless they update both the daily architecture folder for the current date and the central overview pointer. CI also ensures CHANGELOG entries are present.

5. Codex Workflow Enforcement
	Codex (and any agent) must update the current day’s architecture notes and changelog before finishing a task. Skipping this step is disallowed and should be enforced via CI checks plus code review gates.

6. Daily Architecture File Policy
	All architecture content lives exclusively under `docs/architecture/YYYY-MM-DD/`. Each folder must contain, at minimum, that day’s `ARCHITECTURE.md` and daily `CHANGELOG.md`. Root-level or free-floating architecture documents are forbidden; `master_index.md` and `PROJECT_OVERVIEW.md` are the only root navigators and must link to the latest dated folder. CI must fail if architecture notes appear outside the daily directories or if the current day’s folder is missing required files.

## End-of-Session Checklist (for humans & agents)

Before any session is considered complete:
- [ ] Today’s `docs/architecture/YYYY-MM-DD/ARCHITECTURE.md` captures design decisions, benchmarks, and alternatives.
- [ ] Today’s `docs/architecture/YYYY-MM-DD/CHANGELOG.md` lists concrete code/doc changes.
- [ ] Root `CHANGELOG.md` records notable impacts.
- [ ] “You Are Here” in `PROJECT_OVERVIEW.md` (date, finished, in-progress, next steps) is up to date.
- [ ] `master_index.md` points to any new directories/files created today.
- [ ] Tests/CI relevant to the change have run (and pass) or failures are documented.
- [ ] Changes are committed with a message like `Session end: YYYY-MM-DD - <summary>`.

## Verification (post-edit)

- Run `cargo fmt --all` (or language-appropriate formatter) to ensure no accidental formatting drift.
- Validate internal Markdown links (e.g., with `lychee --offline` or equivalent) after documentation edits.
- Confirm no TODO/FIXME comments were introduced unless explicitly requested.
- Summarize changes (file names + line ranges) in the PR or session notes so reviewers can trace updates quickly.
