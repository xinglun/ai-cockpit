#!/usr/bin/env python3
"""Record coordination-cost observations without making an unproven benefit claim."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import time
from pathlib import Path


def sample(repo: Path, label: str, binary: Path | None) -> dict:
    started = time.perf_counter_ns()
    command = None
    state = "unavailable"
    reason = "candidate Runtime binary was not supplied"
    returncode = None
    if binary is not None and binary.is_file():
        command = [str(binary), "work-item", "coordination", "inspect", "--repo", str(repo)]
        result = subprocess.run(command, cwd=repo, capture_output=True, text=True, check=False)
        returncode = result.returncode
        state = "observed" if result.returncode == 0 else "failed"
        reason = None if state == "observed" else result.stderr.strip() or "inspect failed"
    elapsed_ms = (time.perf_counter_ns() - started) / 1_000_000
    coordination_root = repo / ".git" / ".ai-cockpit" / "coordination"
    files = sum(1 for path in coordination_root.rglob("*") if path.is_file()) if coordination_root.is_dir() else 0
    return {
        "label": label,
        "state": state,
        "elapsedMs": elapsed_ms,
        "command": command,
        "returnCode": returncode,
        "coordinationFileCount": files,
        "processCount": 0 if command is None else 1,
        "temporaryDiskBytes": 0,
        "unavailableReason": reason,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--binary", type=Path)
    args = parser.parse_args()
    binary = args.binary
    if binary is None:
        candidate = os.environ.get("AI_COCKPIT_BIN")
        binary = Path(candidate) if candidate else None
    samples = [sample(args.repo, label, binary) for label in ("cold", "first", "warm")]
    print(json.dumps({
        "schemaVersion": 1,
        "repository": str(args.repo.resolve()),
        "runtime": str(binary) if binary else None,
        "samples": samples,
        "baselineComparison": {
            "state": "unavailable",
            "reason": "no comparable pre-collaboration baseline was supplied",
        },
        "benefitClaim": "not_claimed",
    }, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
