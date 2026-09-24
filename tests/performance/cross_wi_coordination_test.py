#!/usr/bin/env python3
"""Contract tests for the raw, no-benefit-claim coordination probe."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


def main() -> None:
    repository = Path(__file__).resolve().parents[2]
    probe = repository / "tests" / "performance" / "cross_wi_coordination.py"
    result = subprocess.run(
        [sys.executable, str(probe), "--repo", str(repository)],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    report = json.loads(result.stdout)
    assert report["schemaVersion"] == 1
    assert report["baselineComparison"]["state"] == "unavailable"
    assert report["benefitClaim"] == "not_claimed"
    assert [sample["label"] for sample in report["samples"]] == ["cold", "first", "warm"]
    for sample in report["samples"]:
        assert "elapsedMs" in sample
        assert "coordinationFileCount" in sample
        assert "processCount" in sample
        assert "temporaryDiskBytes" in sample
        if sample["state"] == "unavailable":
            assert sample["unavailableReason"]


if __name__ == "__main__":
    main()
