#!/usr/bin/env python3
"""Verify perf smoke test timing metrics from log file."""

import sys
import pathlib

def main():
    log_path = pathlib.Path(sys.argv[1])

    if not log_path.exists():
        sys.exit(f"Log file not found: {log_path}")

    lines = [
        line.strip()
        for line in log_path.read_text().splitlines()
        if "timing|phase=codex_job" in line
    ]

    if not lines:
        sys.exit("No timing|phase=codex_job lines found")

    latest = lines[-1]
    parts = {}
    for chunk in latest.split("|"):
        if "=" in chunk:
            key, value = chunk.split("=", 1)
            parts[key] = value

    try:
        total_ms = float(parts.get("total_ms", "0"))
    except ValueError:
        sys.exit(f"Invalid total_ms in timing log: {latest}")

    if total_ms > 2000:
        sys.exit(f"Codex job exceeded SLA: total_ms={total_ms}")

    pty_attempted = parts.get("pty_attempted", "false").lower() == "true"
    if pty_attempted:
        sys.exit("perf_smoke job unexpectedly attempted PTY")

    print(f"Perf smoke metrics valid: {latest}")

if __name__ == "__main__":
    main()
