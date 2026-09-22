#!/usr/bin/env python3
"""Keep the public repository entry point small and route work by task."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
GUIDES = {
    "ordinary-work-item": "agents/skills/ordinary-work-item.md",
    "verification-failure-recovery": "agents/skills/verification-failure-recovery.md",
    "provider-resource-finalization": "agents/skills/provider-resource-finalization.md",
    "release-upgrade-acceptance": "agents/skills/release-upgrade-acceptance.md",
}
REQUIRED_HEADINGS = (
    "## Applicability",
    "## Authoritative inputs",
    "## Operations",
    "## Success conditions",
    "## Failure evidence",
    "## Continue or stop",
)


def main() -> None:
    for guide_id, relative_path in GUIDES.items():
        path = ROOT / relative_path
        assert path.is_file(), f"missing guide {guide_id}: {path}"
        text = path.read_text(encoding="utf-8")
        for heading in REQUIRED_HEADINGS:
            assert heading in text, f"{guide_id} lacks {heading}"

    agents = (ROOT / "AGENTS.md").read_text(encoding="utf-8")
    for relative_path in GUIDES.values():
        assert relative_path in agents, f"AGENTS.md does not link {relative_path}"
    assert "Runtime output is authoritative for action admission" in agents
    assert "resource-bound Work Item uses latest remote default base" not in agents
    assert "for a no-resource Work Item, after removing its exact branch/worktree" not in agents
    assert "ordinary-work-item" in agents
    assert "`release-upgrade-acceptance` only" in agents
    assert "`provider-resource-finalization` only" in agents

    readme = (ROOT / ".ai/README.md").read_text(encoding="utf-8")
    for command in ("inspect --repo", "status --repo", "doctor --repo"):
        assert command in readme, f".ai/README.md lacks {command}"
    assert "../agents/skills/ordinary-work-item.md" in readme

    route = agents.split("## Task guide routing", 1)[1].split("## ", 1)[0]
    assert "ordinary-work-item" in route
    assert "release-upgrade-acceptance" in route
    assert "provider-resource-finalization" in route


if __name__ == "__main__":
    main()
