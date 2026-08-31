#!/usr/bin/env python3
"""Validate the repository's local-only reference-source binding.

The current reference checkout is deliberately operator-owned.  This helper
validates the small lock file everywhere and, when a checkout is supplied,
verifies its exact Git identity and clean state.  It never clones, fetches, or
otherwise accesses a network source.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import NoReturn


COMMIT_RE = re.compile(r"^[0-9a-f]{40}$")
ENV_RE = re.compile(r"^[A-Z][A-Z0-9_]*$")
REQUIRED_KEYS = {"schema", "source", "path_env", "commit", "network_access"}


def fail(message: str) -> NoReturn:
    raise ValueError(message)


def parse_lock(path: Path) -> dict[str, str]:
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError as error:
        fail(f"cannot read reference source lock: {error}")
    values: dict[str, str] = {}
    for line_number, raw_line in enumerate(lines, start=1):
        line = raw_line.split("#", 1)[0].strip()
        if not line:
            continue
        key, separator, raw_value = line.partition("=")
        if not separator:
            fail(f"lock line {line_number} is not key=value")
        key = key.strip()
        value = raw_value.strip()
        if value.startswith('"') and value.endswith('"'):
            value = value[1:-1]
        elif value not in {"true", "false"} and not value.isdigit():
            fail(f"lock line {line_number} has an unsupported value")
        if key in values:
            fail(f"reference source lock repeats {key}")
        values[key] = value
    missing = REQUIRED_KEYS - values.keys()
    if missing:
        fail("reference source lock is missing: " + ", ".join(sorted(missing)))
    if values["schema"] != "1":
        fail("reference source lock schema must be 1")
    if values["source"] != "local-git-checkout":
        fail("reference source must be local-git-checkout")
    if not ENV_RE.fullmatch(values["path_env"]):
        fail("reference source path_env must be an uppercase environment name")
    if not COMMIT_RE.fullmatch(values["commit"]):
        fail("reference source commit must be a 40-character lowercase SHA")
    if values["network_access"] != "false":
        fail("reference source network_access must be false")
    return values


def git_output(root: Path, *args: str) -> str:
    try:
        result = subprocess.run(
            ["git", "-C", str(root), *args],
            check=True,
            capture_output=True,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError) as error:
        detail = getattr(error, "stderr", "") or str(error)
        fail(f"reference checkout git check failed: {detail.strip()}")
    return result.stdout.strip()


def check_checkout(root: Path, expected_commit: str) -> None:
    root = root.expanduser().resolve()
    if not root.is_dir():
        fail("reference checkout does not exist")
    git_marker = root / ".git"
    if not git_marker.is_dir() and not git_marker.is_file():
        fail("reference path is not a Git checkout")
    actual_commit = git_output(root, "rev-parse", "HEAD")
    if actual_commit != expected_commit:
        fail("reference checkout HEAD does not match the locked commit")
    if git_output(root, "status", "--porcelain"):
        fail("reference checkout is not clean")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--lock", type=Path, required=True)
    parser.add_argument("--reference", type=Path)
    args = parser.parse_args()
    try:
        lock = parse_lock(args.lock)
        reference = args.reference
        if reference is None:
            configured = os.environ.get(lock["path_env"])
            if configured:
                reference = Path(configured)
        if reference is not None:
            check_checkout(reference, lock["commit"])
        payload = {
            "ok": True,
            "source": lock["source"],
            "commit": lock["commit"],
            "networkAccess": False,
            "referenceChecked": reference is not None,
        }
        print(json.dumps(payload, sort_keys=True))
        return 0
    except ValueError as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
