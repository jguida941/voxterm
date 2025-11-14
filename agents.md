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
  - Voice processing (capture + STT) should target <750ms on CI hardware for short utterances; <2s is acceptable with good UX.
  - Total voice→Codex round-trip includes external Codex API latency (5-30s typical) which is not under wrapper control.
  - Voice pipeline must comply with `docs/audits/latency_remediation_plan_2025-11-12.md` and the corrected Phase 2B design (`docs/architecture/2025-11-13/PHASE_2B_CORRECTED_DESIGN.md`).
  - Implementation requires: non-blocking audio callback, streaming mel + Whisper FFI OR cloud STT, bounded queues with drop-oldest backpressure, graded fallback ladder (streaming → batch → manual), and per-request latency metrics with CI gates.
  - Investigate race conditions, async vs sync boundaries, blocking file/log I/O, misconfigured channels, cross-thread contention, and unintended Python fallbacks.
  - Provide diagnostic logs and a performance trace for each call path.
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

## Codex Integration & UX Parity (Hard Requirements)

- **Strict superset**: The wrapper must expose everything the native Codex client/CLI can do (all `/` commands, multi-step conversations, tool invocations, streaming/thinking indicators, workspace/file operations, etc.) plus additional modalities such as voice and future orchestration helpers. No Codex capability may be removed or degraded.
- **Codex as source of truth**: Do not reimplement or fork Codex features with divergent semantics. Commands like `/edit`, `/undo`, `/explain`, `/files`, `/stack`, etc. must be forwarded directly to Codex’s real interfaces; the wrapper may decorate or extend behavior but never change outcomes silently.
- **Backend abstraction**: Implement and maintain a `CodexBackend` interface that defines operations such as `send_prompt`, `send_slash_command`, `stream_tokens`, `list_files`, and related workspace actions. All UI/voice code interacts only with `CodexBackend`, not raw CLI stdout.
- **Multiple backends**: Default backend is the existing PTY/CLI (driving the `codex` binary). Designs must be compatible with a future HTTP/WebSocket backend that talks to the official API without rewriting UI/voice layers.
- **Slash command routing**: The input layer must parse `/` prefixes, map them to typed `Command` variants, and dispatch through `CodexBackend`. Streaming responses must emit incremental events so the TUI can show “thinking…” state and live tokens.
- **Working directory control**: Provide configuration (and smart defaults) for Codex’s working directory. Auto-detect a project root (e.g., nearest `.git`) when unset so Codex operations always run in the expected workspace.

	⸻

## Wrapper Scope Correction Instruction (Paste Before Work)

> **INSTRUCTION TO CODEX — WRAPPER SCOPE CORRECTION**  
> 1. **Target = Codex UX parity + extras**  
>    - Everything the Codex client/CLI can do today (all `/` commands, multi-step conversations, tool integrations, streaming/thinking indicators, workspace/file ops, etc.) **must** work through this wrapper. Voice and future orchestration features are additional layers, not replacements.  
> 2. **Codex is the source of truth**  
>    - Do **not** re-implement or fork Codex features. `/edit`, `/undo`, `/explain`, `/files`, `/stack`, etc. must forward to Codex’s real interfaces with identical semantics.  
> 3. **Backend abstraction, not hard-wired CLI**  
>    - Implement a `CodexBackend` trait with methods such as `send_message`, `send_slash_command`, `stream_tokens`, `list_files`, etc. All UI/voice code must depend only on this trait. Support both the PTY/CLI backend today and a future HTTP/WebSocket backend without rewriting UI layers.  
> 4. **Slash commands & streaming UX**  
>    - Input must detect `/` commands, map them to typed enums, and dispatch via the backend. Backends emit streaming events so the TUI can show “thinking…” state and incremental tokens.  
> 5. **Working directory / project context**  
>    - Expose configuration (and auto-detect a `.git` root when unset) for Codex’s working directory so all commands operate in the correct repo.  
> 6. **Plan before code (per AgentMD)**  
>    - Before coding: read AgentMD + relevant design docs, propose 2–3 architectural approaches if choices exist, document the design in `docs/architecture/YYYY-MM-DD/ARCHITECTURE.md`, and wait for approval. After approval, implement the backend abstraction, routed slash commands, streaming indicators, and tests proving the routing works end-to-end. No coding begins without this plan/approval cycle.

Use this block verbatim before starting any work on Codex integration to ensure scope cannot be down-scoped.

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
