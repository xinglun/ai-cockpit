#!/usr/bin/env python3
"""Project the protocol-owned interface description into reference pages.

The Rust CLI remains the source of the description.  This helper only replaces
the marked generated region and never touches surrounding, human-authored
documentation.  ``--check`` is side-effect free and is the CI/docs gate.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


BEGIN = "<!-- AI_COCKPIT_INTERFACE_FACTS:BEGIN work-item-outcome -->"
END = "<!-- AI_COCKPIT_INTERFACE_FACTS:END work-item-outcome -->"

REFERENCES = (
    ("docs/reference/commands.md", "en"),
    ("docs/reference/commands.zh-CN.md", "zh-CN"),
    ("docs/reference/commands.ja.md", "ja"),
)


def interface_region(document: str) -> tuple[int, int, str]:
    start = document.find(BEGIN)
    if start < 0:
        raise ValueError(f"missing generated-region begin marker: {BEGIN}")
    end_marker = document.find(END, start)
    if end_marker < 0:
        raise ValueError(f"missing generated-region end marker: {END}")
    end = end_marker + len(END)
    return start, end, document[start:end]


def render(repo: Path, language: str) -> str:
    command = [
        "cargo",
        "run",
        "--quiet",
        "--locked",
        "--offline",
        "-p",
        "cockpit-cli",
        "--",
        "capability",
        "show",
        "--repo",
        str(repo),
        "--surface",
        "work-item-outcome",
        "--format",
        "markdown",
        "--language",
        language,
    ]
    result = subprocess.run(
        command,
        cwd=repo,
        check=False,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip()
        raise RuntimeError(
            f"interface description command failed for {language}"
            + (f": {detail}" if detail else "")
        )
    return result.stdout.rstrip("\n")


def check_or_write(repo: Path, check_only: bool) -> int:
    drift = False
    for relative_path, language in REFERENCES:
        path = repo / relative_path
        document = path.read_text(encoding="utf-8")
        start, end, actual = interface_region(document)
        expected = render(repo, language)
        if actual != expected:
            drift = True
            if check_only:
                print(f"generated interface facts drift: {relative_path}", file=sys.stderr)
            else:
                path.write_text(
                    document[:start] + expected + document[end:],
                    encoding="utf-8",
                    newline="",
                )
                print(f"updated {relative_path}")
    if check_only and drift:
        return 1
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, required=True)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="fail when generated regions drift")
    mode.add_argument("--write", action="store_true", help="update only the generated regions")
    args = parser.parse_args()
    check_only = not args.write
    try:
        return check_or_write(args.repo.resolve(), check_only)
    except (OSError, RuntimeError, ValueError) as error:
        print(f"interface reference generation failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
