#!/usr/bin/env python3
"""Convenience launcher for the Rust TUI on macOS (or any POSIX system)."""

import argparse
import os
import subprocess
import sys
from pathlib import Path


def detect_repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def detect_default_model(repo_root: Path) -> Path | None:
    candidate = repo_root / "models" / "ggml-base.en.bin"
    return candidate if candidate.exists() else None


def build_command(args: argparse.Namespace, model_path: Path | None) -> list[str]:
    cmd = ["cargo", "run"]
    if args.release:
        cmd.append("--release")
    cmd.append("--")
    if model_path:
        cmd.extend(["--whisper-model-path", str(model_path)])
    cmd.extend(args.tui_args)
    return cmd


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Launch the Rust Codex Voice TUI with sensible defaults."
    )
    parser.add_argument(
        "-m",
        "--model-path",
        type=Path,
        help="Path to a ggml*.bin model (defaults to models/ggml-base.en.bin if present).",
    )
    parser.add_argument(
        "-r",
        "--release",
        action="store_true",
        help="Run the release build of rust_tui.",
    )
    parser.add_argument(
        "tui_args",
        nargs=argparse.REMAINDER,
        help="Extra args forwarded to rust_tui (use -- to separate).",
    )
    args = parser.parse_args()

    repo_root = detect_repo_root()
    rust_tui_dir = repo_root / "rust_tui"
    if not rust_tui_dir.exists():
        print(f"error: expected rust_tui directory at {rust_tui_dir}", file=sys.stderr)
        return 1

    model_path = args.model_path
    if model_path is not None:
        model_path = model_path.expanduser().resolve()
        if not model_path.exists():
            print(
                f"error: specified model path does not exist: {model_path}",
                file=sys.stderr,
            )
            return 1
        if not model_path.is_file():
            print(
                f"error: specified model path is not a file: {model_path}",
                file=sys.stderr,
            )
            return 1
    if model_path is None:
        model_path = detect_default_model(repo_root)

    if model_path is None:
        print(
            "warning: no --model-path provided and models/ggml-base.en.bin not found; "
            "rust_tui may fall back to the Python pipeline.",
            file=sys.stderr,
        )

    tui_args = args.tui_args[1:] if args.tui_args and args.tui_args[0] == "--" else args.tui_args
    args = argparse.Namespace(**{**vars(args), "tui_args": tui_args})
    cmd = build_command(args, model_path)

    env = os.environ.copy()

    try:
        subprocess.run(cmd, cwd=rust_tui_dir, check=True, env=env)
    except subprocess.CalledProcessError as exc:
        return exc.returncode
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
