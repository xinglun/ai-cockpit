#!/usr/bin/env python3
"""Guard the ordinary reader route's measured documentation cost."""

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
DEFAULT_READ_SET = (
    Path("AGENTS.md"),
    Path(".ai/README.md"),
    Path(".ai/agent-interface.json"),
    Path("agents/skills/README.md"),
    Path("agents/skills/ordinary-work-item.md"),
)
BASELINE_BYTES = 22_081
MAXIMUM_BYTES = 13_248
GUIDES = (
    "ordinary-work-item",
    "verification-failure-recovery",
    "provider-resource-finalization",
    "release-upgrade-acceptance",
)


def main() -> None:
    sizes = {str(path): (ROOT / path).stat().st_size for path in DEFAULT_READ_SET}
    total = sum(sizes.values())
    reduction = BASELINE_BYTES - total
    reduction_percent = reduction / BASELINE_BYTES * 100

    assert total <= MAXIMUM_BYTES, (
        f"ordinary default read set grew beyond the budget: {total} > {MAXIMUM_BYTES}"
    )
    assert total < BASELINE_BYTES, "the default read set did not shrink"

    ordinary = (ROOT / "agents/skills/ordinary-work-item.md").read_text(
        encoding="utf-8"
    ).lower()
    for forbidden in ("provider", "release", "upgrade"):
        assert forbidden not in ordinary, (
            f"ordinary guide contains conditional {forbidden} guidance"
        )

    index = (ROOT / "agents/skills/README.md").read_text(encoding="utf-8")
    for guide_id in GUIDES:
        guide_path = ROOT / "agents/skills" / f"{guide_id}.md"
        assert guide_path.is_file(), f"missing task guide: {guide_path}"
        assert guide_id in index, f"guide is not discoverable: {guide_id}"

    print(json.dumps(
        {
            "baselineBytes": BASELINE_BYTES,
            "currentBytes": total,
            "reductionBytes": reduction,
            "reductionPercent": round(reduction_percent, 1),
            "files": sizes,
        },
        sort_keys=True,
    ))


if __name__ == "__main__":
    main()
