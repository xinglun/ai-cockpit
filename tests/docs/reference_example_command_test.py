#!/usr/bin/env python3
"""Check current reader-route command examples against the actual CLI help."""

from __future__ import annotations

import os
import shlex
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
ROUTE_FILES = (
    ROOT / "AGENTS.md",
    ROOT / ".ai/README.md",
    ROOT / "agents/skills/README.md",
    ROOT / "docs/getting-started/README.md",
    ROOT / "docs/getting-started/README.zh-CN.md",
    ROOT / "docs/getting-started/README.ja.md",
)


def cli_prefix() -> list[str]:
    configured = os.environ.get("AI_COCKPIT_INTERFACE_DESCRIPTION_BIN")
    candidates = [Path(configured)] if configured else []
    candidates.extend((ROOT / "target/debug/ai-cockpit", ROOT / "target/release/ai-cockpit"))
    for candidate in candidates:
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return [str(candidate)]
    return [
        "cargo",
        "run",
        "--quiet",
        "--locked",
        "--offline",
        "-p",
        "cockpit-cli",
        "--",
    ]


def examples() -> list[tuple[Path, list[str]]]:
    found: list[tuple[Path, list[str]]] = []
    for path in ROUTE_FILES:
        text = path.read_text(encoding="utf-8")
        for line in text.splitlines():
            for fragment in line.split("`"):
                if not fragment.startswith("ai-cockpit"):
                    continue
                tokens = shlex.split(fragment)
                if tokens and tokens[0] == "ai-cockpit":
                    found.append((path, tokens[1:]))
    return found


def main() -> int:
    failures: list[str] = []
    for path, tokens in examples():
        command: list[str] = []
        options: list[str] = []
        index = 0
        while index < len(tokens):
            token = tokens[index]
            if token.startswith("--"):
                options.append(token.split("=", 1)[0])
                if "=" not in token and index + 1 < len(tokens) and not tokens[index + 1].startswith("--"):
                    index += 1
            elif token.startswith("<") or token.startswith("["):
                pass
            else:
                command.append(token)
            index += 1

        result = subprocess.run(
            cli_prefix() + command + ["--help"],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        help_text = result.stdout + result.stderr
        label = f"{path.relative_to(ROOT)}: ai-cockpit {' '.join(tokens)}"
        if result.returncode != 0:
            failures.append(f"{label}: help failed: {help_text.strip()}")
            continue
        for option in options:
            if option not in help_text:
                failures.append(f"{label}: option {option} is absent from current help")

    if failures:
        raise SystemExit("\n".join(failures))
    print(f"validated {len(examples())} current command examples")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
