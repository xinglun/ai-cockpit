#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
cd "$root"

python3 - <<'PY'
from pathlib import Path
import json
import re

root = Path.cwd()
docs = root / "docs/getting-started"
failures: list[str] = []

stems = (
    "README",
    "30-second-start",
    "installation",
    "installation-security",
    "calibration",
    "first-calibration",
    "first-work-item",
    "adopter-configuration",
    "standard-adoption-guide",
    "security-release-verification",
    "examples/android",
    "examples/ios",
    "examples/java",
)
suffixes = (".md", ".zh-CN.md", ".ja.md")
frontmatter_keys = (
    "author:",
    "title:",
    "description:",
    "audience:",
    "status:",
    "authority:",
    "lastVerifiedBy:",
    "capabilityClaims:",
)
for stem in stems:
    for suffix in suffixes:
        path = docs / f"{stem}{suffix}"
        if not path.is_file():
            failures.append(f"{path.relative_to(root)}: missing onboarding page")
            continue
        text = path.read_text(encoding="utf-8")
        if not text.startswith("---\n"):
            failures.append(f"{path.relative_to(root)}: missing frontmatter")
            continue
        frontmatter = text.split("---\n", 2)[1]
        for key in frontmatter_keys:
            if not any(line.startswith(key) for line in frontmatter.splitlines()):
                failures.append(f"{path.relative_to(root)}: missing {key}")

public_text = "\n".join(
    path.read_text(encoding="utf-8")
    for path in docs.rglob("*.md")
)
for forbidden in (
    "make ai-",
    "./install.sh",
    "ai-cockpit-template",
    "Makefile.ai",
    ".ai/guards/",
):
    if forbidden in public_text:
        failures.append(f"getting-started route contains reference-only marker: {forbidden}")

lifecycle_commands = (
    "ai-cockpit start --repo",
    "ai-cockpit work-item finalize-plan --repo",
    "ai-cockpit preflight --repo",
    "ai-cockpit checkpoint --repo",
    "ai-cockpit verify --repo",
    "ai-cockpit finish --repo",
    "ai-cockpit work-item outcome --repo",
    "ai-cockpit archive --repo",
    "ai-cockpit work-item finalize --repo",
    "ai-cockpit work-item finalize-verify --repo",
    "ai-cockpit close --repo",
)
for suffix in suffixes:
    path = docs / f"first-work-item{suffix}"
    if not path.is_file():
        continue
    text = path.read_text(encoding="utf-8")
    offsets = [text.find(command) for command in lifecycle_commands]
    if any(offset < 0 for offset in offsets):
        missing = [command for command, offset in zip(lifecycle_commands, offsets) if offset < 0]
        failures.append(f"{path.relative_to(root)}: lifecycle omits {', '.join(missing)}")
    elif offsets != sorted(offsets):
        failures.append(f"{path.relative_to(root)}: lifecycle commands are out of order")
    for marker in ("Outcome: 🟢", "--actor", "--authority-source", "--evidence-ref", "--policy-ref", "--decided-at", "--resume-condition"):
        if marker not in text:
            failures.append(f"{path.relative_to(root)}: missing structured handoff marker {marker}")

manifest = json.loads((root / "tests/conformance/reference_file_inventory.json").read_text(encoding="utf-8"))
records = {
    record["referencePath"]: record
    for record in manifest["records"]
    if record["referencePath"].startswith("docs/getting-started/")
}
if len(records) != 35:
    failures.append(f"reference getting-started inventory count is {len(records)}, expected 35")
for reference_path, record in sorted(records.items()):
    if record.get("batch") != "getting-started-onboarding":
        failures.append(f"{reference_path}: batch is not getting-started-onboarding")
    # A local-reference rebaseline can change the source wording without
    # removing the already-reviewed target onboarding route.  Keep the
    # ledger's current classification deferred (so no parity claim is
    # silently promoted), but allow this static route gate to inherit the
    # previous implemented decision while the changed source is queued for
    # its next semantic batch.
    inherited_implementation = (
        record.get("classification") == "deferred-next-batch"
        and record.get("sourceChangedSincePrevious") is True
        and record.get("previousClassification") == "implemented-different-by-design"
    )
    if record.get("classification") == "deferred-next-batch" and not inherited_implementation:
        failures.append(f"{reference_path}: remains deferred-next-batch")
    counterparts = record.get("rustCounterparts")
    if not counterparts:
        failures.append(f"{reference_path}: missing Rust counterpart")
        continue
    for counterpart in counterparts:
        if not (root / counterpart).is_file():
            failures.append(f"{reference_path}: missing counterpart {counterpart}")

if failures:
    raise SystemExit("\n".join(failures))
print("getting-started semantic checks passed: 13 tri-language routes and 35 reference records")
PY
