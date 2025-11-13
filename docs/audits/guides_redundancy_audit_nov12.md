# Documentation Redundancy Audit & Restructuring Proposal

**Date**: 2025-11-12
**Issue**: `docs/guides/` has become bloated with 1,134 lines across 5 files that overlap heavily and don't align with agents.md SDLC requirements.

> **Status (2025-11-12)**: The consolidation plan below has now been executed. Guides moved into `docs/references/`, obsolete copies were archived, and the new quick start/testing references track the verified commands.

---

## Problem Analysis

### Current docs/guides/ Inventory

| File | Lines | Purpose | Issue |
|------|-------|---------|-------|
| `architecture_overview.md` | 415 | System diagrams, file structure, commands | ❌ **DUPLICATE** of root info + daily arch notes |
| `master_doc.md` | 115 | "High-level entry point" | ❌ **REDUNDANT** — PROJECT_OVERVIEW.md does this |
| `DEVELOPER_GUIDE.md` | 144 | Command reference, testing | ⚠️ **PARTIALLY USEFUL** — has unique Python commands |
| `plan.md` | 395 | Original MVP roadmap | ✅ **ARCHIVED** (already marked legacy) |
| `HOW_TO_TEST_AUDIO.md` | 65 | Audio testing steps | ✅ **USEFUL** — keep as-is |

**Total**: 1,134 lines (829 lines if we exclude archived `plan.md`)

### agents.md Conflicts

**agents.md line 197**: "Root-level or free-floating architecture documents are forbidden; master_index.md and PROJECT_OVERVIEW.md are the only root navigators"

**Reality**:
- `docs/guides/architecture_overview.md` (415 lines) — **IS** a free-floating architecture doc
- `docs/guides/master_doc.md` (115 lines) — Duplicates PROJECT_OVERVIEW.md purpose
- These files **compete** with the daily architecture folders

### The Redundancy Matrix

| Content Type | Where It Lives NOW | Where It SHOULD Live |
|--------------|-------------------|---------------------|
| System architecture diagrams | `guides/architecture_overview.md` | ❌ Belongs in `docs/architecture/YYYY-MM-DD/` |
| Quick start commands | `guides/master_doc.md` + `architecture_overview.md` | ✅ README.md (root) |
| SDLC roadmap | `PROJECT_OVERVIEW.md` + `guides/master_doc.md` | ✅ PROJECT_OVERVIEW.md only |
| Daily design decisions | `docs/architecture/YYYY-MM-DD/` | ✅ Correct |
| Testing procedures | `guides/HOW_TO_TEST_AUDIO.md` + `DEVELOPER_GUIDE.md` | ⚠️ Consolidate into ONE |
| Python pipeline commands | `guides/DEVELOPER_GUIDE.md` | ✅ Keep (legacy reference) |
| Historical roadmap | `guides/plan.md` | ✅ Already marked legacy |

---

## Proposed Solution: 3-Tier Documentation Hierarchy

### Tier 1: ROOT NAVIGATION (Stable, Updated Per-Session)

**Update cadence**: End of work session (weekly or when major milestone completes)

```
/ (root)
├── README.md              ← Quick start (5-min onboard)
├── PROJECT_OVERVIEW.md    ← Current roadmap + latest arch pointer
├── master_index.md        ← Navigation hub
├── agents.md              ← SDLC requirements
└── CHANGELOG.md           ← Repository-wide changes
```

**Purpose**:
- **README.md**: Gets someone running in 5 minutes
- **PROJECT_OVERVIEW.md**: Current mission + link to latest daily folder
- **master_index.md**: Find everything
- **agents.md**: How we work (governance)
- **CHANGELOG.md**: What changed (releases)

**Rule**: These are **INDEXES ONLY**, not detailed docs

---

### Tier 2: DAILY ARCHITECTURE (Detailed, Updated Daily)

**Update cadence**: Every working day

```
docs/architecture/
├── 2025-11-11/
│   ├── ARCHITECTURE.md    ← Design decisions, alternatives, benchmarks
│   ├── CHANGELOG.md       ← Daily incremental changes
│   └── diagrams/          ← Supporting visuals
└── 2025-11-12/
    ├── ARCHITECTURE.md
    ├── CHANGELOG.md
    └── diagrams/
```

**Purpose**: Full traceability of design decisions, as required by agents.md

**Content that belongs here**:
- System architecture diagrams (currently in `guides/architecture_overview.md`)
- Component breakdowns (currently in `guides/architecture_overview.md`)
- Design alternatives considered
- Performance benchmarks
- "Why we chose X over Y" rationale

**Rule**: This is the **AUTHORITATIVE** architecture source

---

### Tier 3: LIVING REFERENCES (Stable, Updated As-Needed)

**Update cadence**: When procedures change

```
docs/references/          ← NEW directory name (not "guides")
├── quick_start.md        ← 5-min minimal setup
├── testing.md            ← How to test (audio, unit, integration)
├── python_legacy.md      ← Old Python pipeline commands (for reference)
└── troubleshooting.md    ← Common issues + fixes
```

**Purpose**: Operational how-to docs that don't change often

**Rule**: These are **PROCEDURES**, not architecture

---

## Specific File Actions

### ❌ DELETE (Redundant)

1. **`docs/guides/master_doc.md`** (115 lines)
   - **Reason**: 100% redundant with `PROJECT_OVERVIEW.md` + `README.md`
   - **Migration**: Nothing to save, already covered

2. **`docs/guides/architecture_overview.md`** (415 lines)
   - **Reason**: Violates agents.md (free-floating arch doc) + overlaps daily arch folders
   - **Migration**:
     - System diagrams → Move to `docs/architecture/2025-11-12/diagrams/`
     - Component descriptions → Already in dated ARCHITECTURE.md files
     - Quick commands → Already in README.md

### ✅ KEEP & RENAME

3. **`docs/guides/HOW_TO_TEST_AUDIO.md`** (65 lines)
   - **Action**: Move to `docs/references/testing.md`
   - **Reason**: Useful standalone procedural doc

4. **`docs/guides/DEVELOPER_GUIDE.md`** (144 lines)
   - **Action**: Split into:
     - `docs/references/quick_start.md` — Cargo commands, model setup (50 lines)
     - `docs/references/python_legacy.md` — Python pipeline commands (94 lines)
   - **Reason**: Contains unique Python reference + cargo commands not elsewhere

### 📦 ARCHIVE (Historical)

5. **`docs/guides/plan.md`** (395 lines)
   - **Action**: Move to `docs/archive/MVP_PLAN_2024.md`
   - **Reason**: Already marked legacy, keep for history

---

## Proposed New Structure

```
codex_voice/
│
├── README.md                      ← 5-min quick start (NEW or updated)
├── PROJECT_OVERVIEW.md            ← Current roadmap (EXISTS)
├── master_index.md                ← Navigation (EXISTS)
├── agents.md                      ← SDLC rules (EXISTS)
├── CHANGELOG.md                   ← Repo changelog (EXISTS)
│
├── docs/
│   ├── architecture/              ← Daily folders (EXISTS)
│   │   ├── 2025-11-11/
│   │   └── 2025-11-12/
│   │       ├── ARCHITECTURE.md    ← Detailed design
│   │       ├── CHANGELOG.md       ← Daily changes
│   │       └── diagrams/          ← System diagrams (NEW)
│   │
│   ├── references/                ← RENAMED from "guides"
│   │   ├── quick_start.md         ← Cargo, models, CLI (NEW)
│   │   ├── testing.md             ← Audio + unit tests (RENAMED)
│   │   ├── python_legacy.md       ← Old Python cmds (NEW)
│   │   └── troubleshooting.md     ← Common issues (NEW, optional)
│   │
│   ├── audits/                    ← External audits (EXISTS)
│   │   ├── claudeaudit.md
│   │   └── claude_audit_nov12.md
│   │
│   └── archive/                   ← Historical docs (EXISTS)
│       ├── MVP_PLAN_2024.md       ← plan.md renamed
│       ├── ... (17 other archived docs)
│       └── OBSOLETE_GUIDES_2025-11-12/  ← NEW archive folder
│           ├── architecture_overview.md
│           └── master_doc.md
```

**Line count reduction**: 1,134 → ~250 lines (78% reduction)

---

## Rationale: Why This Fixes The Problem

### 1. Aligns with agents.md

✅ **No free-floating architecture docs** — All arch content in `docs/architecture/YYYY-MM-DD/`
✅ **Root is navigation only** — README, PROJECT_OVERVIEW, master_index are indexes
✅ **Daily traceability** — Design decisions in dated folders

### 2. Scales for SDLC

**Daily work**:
- Update `docs/architecture/2025-MM-DD/ARCHITECTURE.md` (detailed notes)
- Update `docs/architecture/2025-MM-DD/CHANGELOG.md` (incremental changes)

**Weekly/session end**:
- Update `PROJECT_OVERVIEW.md` (if roadmap changed)
- Update `CHANGELOG.md` (high-level release notes)

**As-needed**:
- Update `docs/references/*.md` (when procedures change)

### 3. Eliminates Redundancy

| Before | After |
|--------|-------|
| 5 guide files (1,134 lines) | 3 reference files (~250 lines) |
| Architecture in 3 places | Architecture in 1 place (daily folders) |
| 3 "quick start" guides | 1 README.md |
| Unclear which is "current" | Clear hierarchy: root → daily → references |

### 4. Developer Experience

**Newcomer**:
1. Read `README.md` (5 min) → Running code
2. Read `PROJECT_OVERVIEW.md` (2 min) → Understand current focus
3. Read latest `docs/architecture/YYYY-MM-DD/ARCHITECTURE.md` (10 min) → Understand decisions

**Existing contributor**:
1. Check `PROJECT_OVERVIEW.md` → Find latest daily folder
2. Read that day's `ARCHITECTURE.md` → See what changed
3. Consult `docs/references/` → Find testing/troubleshooting procedures

**Agent (Claude/GPT)**:
1. Read `agents.md` → Understand SDLC requirements
2. Check `PROJECT_OVERVIEW.md` → Find latest daily folder
3. Update `docs/architecture/YYYY-MM-DD/` → Log work
4. Update `CHANGELOG.md` → Record changes

---

## Migration Plan

### Phase 1: Restructure (30 minutes)

```bash
# 1. Create new references directory
mkdir -p docs/references

# 2. Split DEVELOPER_GUIDE.md
# (Manual: Extract quick start → docs/references/quick_start.md)
# (Manual: Extract Python commands → docs/references/python_legacy.md)

# 3. Rename and move testing guide
mv docs/guides/HOW_TO_TEST_AUDIO.md docs/references/testing.md

# 4. Archive plan.md
mv docs/guides/plan.md docs/archive/MVP_PLAN_2024.md

# 5. Archive obsolete guides
mkdir -p docs/archive/OBSOLETE_GUIDES_2025-11-12
mv docs/guides/architecture_overview.md docs/archive/OBSOLETE_GUIDES_2025-11-12/
mv docs/guides/master_doc.md docs/archive/OBSOLETE_GUIDES_2025-11-12/
mv docs/guides/DEVELOPER_GUIDE.md docs/archive/OBSOLETE_GUIDES_2025-11-12/

# 6. Remove empty guides directory
rmdir docs/guides

# 7. Move diagrams from architecture_overview.md to daily folder
# (Manual: Extract diagrams → docs/architecture/2025-11-12/diagrams/)
```

### Phase 2: Create Missing Root Docs (30 minutes)

**Create `README.md`** (root):
```markdown
# Codex Voice

Voice-controlled interface for Codex CLI using Rust + Whisper.

## Quick Start (5 minutes)

```bash
# 1. Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Download Whisper model
curl -L "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin" \
     -o models/ggml-base.en.bin

# 3. Build and run
cd rust_tui
cargo run --release -- --whisper-model-path ../models/ggml-base.en.bin
```

## Controls

- `Ctrl+R` — Start voice capture
- `Ctrl+V` — Toggle auto voice mode
- `Enter` — Send prompt to Codex
- `Ctrl+C` — Quit

## Documentation

- **Current roadmap**: [PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md)
- **Navigation**: [master_index.md](master_index.md)
- **Daily architecture notes**: [docs/architecture/](docs/architecture/)
- **References**: [docs/references/](docs/references/)
- **SDLC requirements**: [agents.md](agents.md)

## Status

✅ Working: Native Rust audio pipeline (cpal + whisper-rs)
✅ Working: Persistent Codex PTY sessions
🚧 In progress: Sub-second latency optimizations

See [PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md) for current focus.
```

### Phase 3: Update master_index.md (10 minutes)

Update line 17 to reflect new structure:
```markdown
- Guides live in [`docs/references/`](docs/references/) (operational procedures: quick_start.md, testing.md, python_legacy.md, troubleshooting.md).
```

### Phase 4: Update agents.md (10 minutes)

Add clarification about `docs/references/`:
```markdown
## Documentation Hierarchy

1. **Root Navigation** (stable, updated per-session):
   - README.md, PROJECT_OVERVIEW.md, master_index.md, agents.md, CHANGELOG.md

2. **Daily Architecture** (detailed, updated daily):
   - docs/architecture/YYYY-MM-DD/ARCHITECTURE.md
   - docs/architecture/YYYY-MM-DD/CHANGELOG.md
   - docs/architecture/YYYY-MM-DD/diagrams/

3. **Living References** (stable, updated as-needed):
   - docs/references/*.md (operational procedures)

**Rule**: Architecture content ONLY lives in daily folders. Root files are indexes. References are procedures.
```

---

## Update Cadence Summary

| File/Directory | Update Frequency | Trigger |
|----------------|------------------|---------|
| `README.md` | Rarely | When quick start changes |
| `PROJECT_OVERVIEW.md` | Per-session | When roadmap changes |
| `master_index.md` | Per-session | When new docs added |
| `CHANGELOG.md` | Per-session | At end of work session |
| `docs/architecture/YYYY-MM-DD/` | **DAILY** | Every working day |
| `docs/references/*.md` | As-needed | When procedures change |

**Key insight**: You wanted **daily detail** in dated folders, **periodic summaries** in root. This structure delivers exactly that.

---

## Benefits

### ✅ Solves Your Problems

1. **"guides/ has way too much"** → Reduced 1,134 lines to ~250 lines (78% reduction)
2. **"doesn't align with agents.md"** → Now 100% compliant (no free-floating arch docs)
3. **"redundant"** → Eliminated 3 duplicate files
4. **"need daily info in folders"** → All design decisions in `docs/architecture/YYYY-MM-DD/`
5. **"master stuff in root updated periodically"** → Clear per-session update cadence

### ✅ Developer Benefits

- **5-min onboarding** via README.md
- **Clear hierarchy** (root → daily → references)
- **No confusion** about which doc is "current"
- **Scales indefinitely** (daily folders accumulate without bloat)

### ✅ SDLC Benefits

- **Full traceability** (agents.md compliant)
- **Daily decision log** (required by agents.md line 24)
- **CI enforceable** (check for daily folder existence)
- **No drift** (clear update responsibilities)

---

## Recommendation

**DO THIS**:

1. **Today** (80 minutes):
   - Run migration script (Phase 1: 30 min)
   - Create README.md (Phase 2: 30 min)
   - Update master_index.md (Phase 3: 10 min)
   - Update agents.md (Phase 4: 10 min)

2. **Result**: Clean, scalable, agents.md-compliant documentation structure

3. **Effort**: 80 minutes one-time investment

4. **Payoff**: Never get lost in docs again, agents know exactly where to log work

---

## Appendix: agents.md Compliance Checklist

| Requirement | Before | After |
|-------------|--------|-------|
| Daily architecture folders | ✅ Exists | ✅ Exists |
| No root architecture docs | ❌ 2 violations | ✅ Fixed |
| Root is navigation only | ❌ Mixed | ✅ Fixed |
| CHANGELOG per PR | ✅ Exists | ✅ Exists |
| Daily CHANGELOG | ✅ Exists | ✅ Exists |
| Clear update cadence | ❌ Undefined | ✅ Defined |

**Compliance score**: 67% → 100% ✅

---

**End of Audit**

*Recommendation: Implement restructuring today (80 min)*
*Expected outcome: 78% reduction in guide bloat, 100% agents.md compliance, clear SDLC workflow*
