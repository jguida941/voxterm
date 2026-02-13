"""Shared helpers for process execution, env setup, and output handling."""

import os
import shutil
import subprocess
import time
from pathlib import Path
from typing import List, Optional

from .config import REPO_ROOT, SRC_DIR


def cmd_str(cmd: List[str]) -> str:
    """Render a command list as a printable string."""
    return " ".join(cmd)


def run_cmd(
    name: str,
    cmd: List[str],
    cwd: Optional[Path] = None,
    env: Optional[dict] = None,
    dry_run: bool = False,
) -> dict:
    """Run a command and return timing/exit metadata."""
    start = time.time()
    if dry_run:
        print(f"[dry-run] {name}: {cmd_str(cmd)}")
        return {
            "name": name,
            "cmd": cmd,
            "cwd": str(cwd or REPO_ROOT),
            "returncode": 0,
            "duration_s": 0.0,
            "skipped": True,
        }

    result = subprocess.run(cmd, cwd=cwd, env=env)
    duration = time.time() - start
    return {
        "name": name,
        "cmd": cmd,
        "cwd": str(cwd or REPO_ROOT),
        "returncode": result.returncode,
        "duration_s": round(duration, 2),
        "skipped": False,
    }


def build_env(args) -> dict:
    """Build an environment map honoring offline/cache args."""
    env = os.environ.copy()
    if getattr(args, "offline", False):
        env["CARGO_NET_OFFLINE"] = "true"
    if getattr(args, "cargo_home", None):
        env["CARGO_HOME"] = os.path.expanduser(args.cargo_home)
    if getattr(args, "cargo_target_dir", None):
        env["CARGO_TARGET_DIR"] = os.path.expanduser(args.cargo_target_dir)
    return env


def write_output(content: str, output_path: Optional[str]) -> None:
    """Write output to a file or stdout."""
    if output_path:
        path = Path(output_path)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)
        print(f"Report saved to: {path}")
    else:
        print(content)


def pipe_output(content: str, pipe_command: Optional[str], pipe_args: Optional[List[str]]) -> int:
    """Pipe content to another CLI that accepts stdin."""
    if not pipe_command:
        return 0
    cmd = [pipe_command] + (pipe_args or [])
    if not shutil.which(cmd[0]):
        print(f"Pipe command not found: {cmd[0]}")
        return 2
    result = subprocess.run(cmd, input=content, text=True)
    return result.returncode


def should_emit_output(args) -> bool:
    """Return True when a report should be emitted."""
    return args.format != "text" or bool(args.output) or bool(getattr(args, "pipe_command", None))


def confirm_or_abort(message: str, assume_yes: bool) -> None:
    """Prompt for confirmation unless assume_yes is set."""
    if assume_yes:
        return
    try:
        reply = input(f"{message} [y/N] ").strip().lower()
    except EOFError:
        print(f"{message} [y/N] <non-interactive input unavailable>")
        print("Aborted. Re-run with --yes for non-interactive usage.")
        raise SystemExit(1)
    if reply not in ("y", "yes"):
        print("Aborted.")
        raise SystemExit(1)


def find_latest_outcomes_file() -> Optional[Path]:
    """Locate the newest outcomes.json under src/mutants.out."""
    output_dir = SRC_DIR / "mutants.out"
    primary = output_dir / "outcomes.json"
    if primary.exists():
        return primary
    candidates = list(output_dir.rglob("outcomes.json"))
    if not candidates:
        return None
    return max(candidates, key=lambda path: path.stat().st_mtime)
