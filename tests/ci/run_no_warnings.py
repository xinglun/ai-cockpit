#!/usr/bin/env python3
"""Run a quality command with complete visible streams and unchanged exit semantics."""
from __future__ import annotations

import subprocess
import sys


def main() -> int:
    command = sys.argv[1:]
    if command[:1] == ["--"]:
        command = command[1:]
    if not command:
        raise SystemExit("a quality command is required")
    return subprocess.run(command, check=False).returncode


if __name__ == "__main__":
    sys.exit(main())
