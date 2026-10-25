#!/usr/bin/env python3
"""Fail quality commands on actual diagnostic warnings, preserving both streams."""
from __future__ import annotations

import re
import subprocess
import sys
import threading
from pathlib import Path

# Diagnostic prefixes, not occurrences of the word in tests or quoted fixtures.
_WARNING = re.compile(
    r"^(?:\s*warning(?:\[[^\]\r\n]+\])?:|\s*WARNING:|::warning(?: |::)|"
    r"(?:\(node:\d+\) )?(?:\[DEP\d+\] )?DeprecationWarning:|"
    r"[^\r\n]+:\d+: (?:DeprecationWarning|FutureWarning|RuntimeWarning):)"
)
_INCOMPLETE = re.compile(r"^\[(?:diagnostic|output) truncated\]$|^Warning: truncated output")
_ANSI = re.compile(r"\x1b\[[0-?]*[ -/]*[@-~]")


def warning_diagnostics(output: str) -> list[str]:
    return [line for raw in output.splitlines()
            if _WARNING.search(line := _ANSI.sub("", raw)) or _INCOMPLETE.search(line)]


def main() -> int:
    args = sys.argv[1:]
    if args[:1] == ["--check-log"]:
        if len(args) != 2:
            raise SystemExit("--check-log requires one complete regular log")
        path = Path(args[1])
        if path.is_symlink() or not path.is_file():
            raise SystemExit("complete regular quality log is required")
        diagnostics = warning_diagnostics(path.read_text(encoding="utf-8", errors="strict"))
        if diagnostics:
            print("quality command emitted a warning or incomplete diagnostic capture", file=sys.stderr)
            print("\n".join(diagnostics), file=sys.stderr)
        return int(bool(diagnostics))
    if args[:1] == ["--"]:
        args = args[1:]
    if not args:
        raise SystemExit("a quality command is required")
    process = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
                               encoding="utf-8", errors="strict")
    diagnostics: list[str] = []
    errors: list[BaseException] = []

    def relay(stream, target):
        try:
            for line in stream:
                target.write(line)
                target.flush()
                diagnostics.extend(warning_diagnostics(line))
        except BaseException as error:
            errors.append(error)
        finally:
            stream.close()

    threads = [threading.Thread(target=relay, args=(process.stdout, sys.stdout)),
               threading.Thread(target=relay, args=(process.stderr, sys.stderr))]
    for thread in threads:
        thread.start()
    code = process.wait()
    for thread in threads:
        thread.join()
    if code != 0:
        return code if code > 0 else 128 - code
    if errors or diagnostics:
        print("quality command has warnings or incomplete output; zero warnings is required", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
