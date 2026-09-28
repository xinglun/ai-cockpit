#!/usr/bin/env python3
"""Guard the ordinary reader route's measured documentation cost."""

import json
import re
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

    before_verification_heading = "## before verification"
    assert before_verification_heading in ordinary, (
        "ordinary guide must require an evidence-reuse decision before verification"
    )
    before_verification = re.sub(
        r"\s+",
        " ",
        ordinary.split(before_verification_heading, 1)[1].split("\n## ", 1)[0],
    )
    for required_rule in (
        "before launching a declared check",
        "`work-item status`",
        "`work-item validate`",
        "inventory its formal receipt",
        "reuse only fresh, complete evidence",
        "runtime's admitted next action",
        "../../docs/reference/agent-workflow.md#verification-evidence-reuse",
    ):
        assert required_rule in before_verification, (
            f"ordinary guide omits required verification entry rule: {required_rule}"
        )

    workflow = (ROOT / "docs/reference/agent-workflow.md").read_text(
        encoding="utf-8"
    ).lower()
    reuse_heading = "### verification evidence reuse"
    assert reuse_heading in workflow, "agent workflow must define the full reuse procedure"
    reuse_section = re.sub(
        r"\s+", " ", workflow.split(reuse_heading, 1)[1].split("\n### ", 1)[0]
    )
    for required_rule in (
        "`evidencefreshness.state=fresh`",
        "contract digest",
        "source snapshot",
        "runtime executable digest",
        "verification plan and target",
        "complete required check/scenario set",
        "same formal receipt bytes",
        "without spawning those checks again",
        "preserve the old receipt",
        "exact current pr head",
        "local receipt never substitutes for hosted evidence",
        "must not launch those package tests a second time",
    ):
        assert required_rule in reuse_section, (
            f"verification reuse guidance omits required rule: {required_rule}"
        )

    gate_manifest = json.loads(
        (ROOT / "tests/ci/repository_gate_manifest.json").read_text(
            encoding="utf-8"
        )
    )
    gate_commands = {
        gate["id"]: gate["command"] for gate in gate_manifest["gates"]
    }
    protected_regressions = {
        "docs_acceptance": "tests/docs/documentation_acceptance.sh",
        "ci_workspace_coverage_regression": "tests/ci/workspace_package_coverage_test.sh",
        "ci_hosted_runtime_verification_regression": "tests/ci/hosted_runtime_verification_test.sh",
    }
    for gate_id, expected_command in protected_regressions.items():
        assert gate_commands.get(gate_id, [None])[0] == expected_command, (
            f"verification evidence reuse regression is not protected by {gate_id}"
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
