#!/usr/bin/env python3
import argparse
import json
import sys
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description="Check mutation score threshold.")
    parser.add_argument(
        "--path",
        default="mutants.out/outcomes.json",
        help="Path to cargo-mutants outcomes.json",
    )
    parser.add_argument(
        "--threshold",
        type=float,
        default=0.80,
        help="Minimum acceptable mutation score (0.0-1.0)",
    )
    args = parser.parse_args()

    outcomes_path = Path(args.path)
    if not outcomes_path.exists():
        print(f"ERROR: outcomes file not found: {outcomes_path}")
        return 2

    with outcomes_path.open("r", encoding="utf-8") as handle:
        data = json.load(handle)

    caught = int(data.get("caught", 0))
    missed = int(data.get("missed", 0))
    timeout = int(data.get("timeout", 0))
    unviable = int(data.get("unviable", 0))

    denom = caught + missed + timeout
    score = 1.0 if denom == 0 else caught / denom

    print(
        "Mutation score: {score:.2%} (caught {caught}, missed {missed}, timeout {timeout}, unviable {unviable})".format(
            score=score,
            caught=caught,
            missed=missed,
            timeout=timeout,
            unviable=unviable,
        )
    )

    if score < args.threshold:
        print(
            "FAIL: mutation score {score:.2%} is below threshold {threshold:.2%}".format(
                score=score, threshold=args.threshold
            )
        )
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
