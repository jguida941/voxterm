#!/usr/bin/env python3
"""
VoxTerm Mutation Testing Helper

Interactive script for running mutation tests on specific modules.
Outputs results in AI-readable format (JSON/markdown).

Usage:
    python3 dev/scripts/mutants.py              # Interactive mode
    python3 dev/scripts/mutants.py --all        # Run all modules
    python3 dev/scripts/mutants.py --module audio  # Specific module
    python3 dev/scripts/mutants.py --list       # List available modules
"""

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path
from datetime import datetime

# Module definitions with their source paths
MODULES = {
    "audio": {
        "desc": "Audio capture, VAD, resampling",
        "files": ["src/audio/**"],
        "timeout": 120,
    },
    "stt": {
        "desc": "Whisper transcription",
        "files": ["src/stt.rs"],
        "timeout": 120,
    },
    "voice": {
        "desc": "Voice capture orchestration",
        "files": ["src/voice.rs"],
        "timeout": 120,
    },
    "config": {
        "desc": "CLI flags and validation",
        "files": ["src/config/**"],
        "timeout": 60,
    },
    "pty": {
        "desc": "PTY session handling",
        "files": ["src/pty_session/**"],
        "timeout": 120,
    },
    "ipc": {
        "desc": "JSON IPC protocol",
        "files": ["src/ipc/**"],
        "timeout": 90,
    },
    "app": {
        "desc": "App state and logging",
        "files": ["src/app/**"],
        "timeout": 90,
    },
    "overlay": {
        "desc": "Overlay binary (main, writer, status)",
        "files": ["src/bin/codex_overlay/**"],
        "timeout": 180,
    },
}

REPO_ROOT = Path(__file__).parent.parent.parent
SRC_DIR = REPO_ROOT / "src"
OUTPUT_DIR = SRC_DIR / "mutants.out"


def list_modules():
    """Print available modules."""
    print("\nAvailable modules for mutation testing:\n")
    print(f"{'Module':<12} {'Description':<40} {'Timeout':<10}")
    print("-" * 62)
    for name, info in MODULES.items():
        print(f"{name:<12} {info['desc']:<40} {info['timeout']}s")
    print()


def select_modules_interactive():
    """Interactive module selection."""
    print("\n=== VoxTerm Mutation Testing ===\n")
    print("Select modules to test (comma-separated numbers, or 'all'):\n")

    module_list = list(MODULES.keys())
    for i, name in enumerate(module_list, 1):
        info = MODULES[name]
        print(f"  {i}. {name:<12} - {info['desc']}")

    print(f"\n  0. ALL modules (slow)")
    print()

    try:
        choice = input("Enter selection: ").strip().lower()
    except (EOFError, KeyboardInterrupt):
        print("\nCancelled.")
        sys.exit(0)

    if choice in ("0", "all"):
        return module_list

    selected = []
    for part in choice.split(","):
        part = part.strip()
        if part.isdigit():
            idx = int(part) - 1
            if 0 <= idx < len(module_list):
                selected.append(module_list[idx])
        elif part in MODULES:
            selected.append(part)

    return selected if selected else module_list[:1]  # Default to first module


def run_mutants(modules, timeout=300):
    """Run cargo mutants on selected modules."""
    # Build file filter args
    file_args = []
    for mod in modules:
        if mod in MODULES:
            for f in MODULES[mod]["files"]:
                file_args.extend(["-f", f])

    if not file_args:
        print("No valid modules selected.")
        return None

    cmd = [
        "cargo", "mutants",
        "--timeout", str(timeout),
        "-o", "mutants.out",
        "--json",
    ] + file_args

    print(f"\nRunning mutation tests on: {', '.join(modules)}")
    print(f"Command: {' '.join(cmd)}\n")
    print("-" * 60)

    os.chdir(SRC_DIR)
    result = subprocess.run(cmd, capture_output=False)

    return result.returncode


def parse_results():
    """Parse mutation testing results."""
    outcomes_file = OUTPUT_DIR / "outcomes.json"

    if not outcomes_file.exists():
        print(f"No results found at {outcomes_file}")
        return None

    with open(outcomes_file) as f:
        data = json.load(f)

    # Count outcomes
    stats = {
        "killed": 0,
        "survived": 0,
        "timeout": 0,
        "unviable": 0,
        "total": 0,
    }

    survived_mutants = []

    for outcome in data.get("outcomes", []):
        stats["total"] += 1
        status = outcome.get("summary", "unknown")

        if status == "Killed":
            stats["killed"] += 1
        elif status == "Survived":
            stats["survived"] += 1
            survived_mutants.append({
                "file": outcome.get("scenario", {}).get("file", "unknown"),
                "line": outcome.get("scenario", {}).get("line", 0),
                "function": outcome.get("scenario", {}).get("function", "unknown"),
                "mutation": outcome.get("scenario", {}).get("mutation", "unknown"),
            })
        elif status == "Timeout":
            stats["timeout"] += 1
        elif status == "Unviable":
            stats["unviable"] += 1

    # Calculate score
    testable = stats["killed"] + stats["survived"]
    score = (stats["killed"] / testable * 100) if testable > 0 else 0

    return {
        "stats": stats,
        "score": score,
        "survived": survived_mutants,
        "timestamp": datetime.now().isoformat(),
    }


def output_results(results, format="markdown"):
    """Output results in specified format."""
    if results is None:
        return

    stats = results["stats"]
    score = results["score"]
    survived = results["survived"]

    if format == "json":
        print(json.dumps(results, indent=2))
        return

    # Markdown format (AI-readable)
    print("\n" + "=" * 60)
    print("MUTATION TESTING RESULTS")
    print("=" * 60)

    print(f"""
## Summary

| Metric | Value |
|--------|-------|
| Score | **{score:.1f}%** |
| Killed | {stats['killed']} |
| Survived | {stats['survived']} |
| Timeout | {stats['timeout']} |
| Unviable | {stats['unviable']} |
| Total | {stats['total']} |

Threshold: 80%
Status: {"PASS" if score >= 80 else "FAIL"}
""")

    if survived:
        print("## Survived Mutants (need better tests)\n")
        print("| File | Line | Function | Mutation |")
        print("|------|------|----------|----------|")
        for m in survived[:20]:  # Limit to 20
            print(f"| {m['file']} | {m['line']} | {m['function']} | {m['mutation'][:50]} |")

        if len(survived) > 20:
            print(f"\n... and {len(survived) - 20} more")

    print()

    # Save to file
    output_file = OUTPUT_DIR / "summary.md"
    with open(output_file, "w") as f:
        f.write(f"# Mutation Testing Results\n\n")
        f.write(f"Generated: {results['timestamp']}\n\n")
        f.write(f"## Score: {score:.1f}%\n\n")
        f.write(f"- Killed: {stats['killed']}\n")
        f.write(f"- Survived: {stats['survived']}\n")
        f.write(f"- Total: {stats['total']}\n")

    print(f"Results saved to: {output_file}")


def main():
    parser = argparse.ArgumentParser(description="VoxTerm Mutation Testing Helper")
    parser.add_argument("--all", action="store_true", help="Test all modules")
    parser.add_argument("--module", "-m", help="Specific module to test")
    parser.add_argument("--list", "-l", action="store_true", help="List available modules")
    parser.add_argument("--json", action="store_true", help="Output as JSON")
    parser.add_argument("--timeout", "-t", type=int, default=300, help="Timeout in seconds")
    parser.add_argument("--results-only", action="store_true", help="Just parse existing results")

    args = parser.parse_args()

    if args.list:
        list_modules()
        return

    if args.results_only:
        results = parse_results()
        output_results(results, "json" if args.json else "markdown")
        return

    # Select modules
    if args.all:
        modules = list(MODULES.keys())
    elif args.module:
        modules = [m.strip() for m in args.module.split(",")]
    else:
        modules = select_modules_interactive()

    print(f"\nSelected modules: {', '.join(modules)}")

    # Run mutation tests
    returncode = run_mutants(modules, args.timeout)

    # Parse and output results
    results = parse_results()
    output_results(results, "json" if args.json else "markdown")

    # Exit with appropriate code
    if results and results["score"] < 80:
        sys.exit(1)


if __name__ == "__main__":
    main()
