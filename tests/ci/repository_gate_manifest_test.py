#!/usr/bin/env python3
import json
import subprocess
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[2]
manifest_path = root / "tests/ci/repository_gate_manifest.json"
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
entries = manifest["gates"]
ids = [entry["id"] for entry in entries]
commands = [tuple(entry["command"]) for entry in entries]
assert ids == sorted(ids), "gate IDs must be deterministic"
assert len(ids) == len(set(ids)), "duplicate gate ID"
assert len(commands) == len(set(commands)), "duplicate gate command"

registered = {entry["command"][0] for entry in entries}
registered.update(path for entry in entries for path in entry.get("covers", []))
required = {
    str(path.relative_to(root))
    for path in (root / "tests").rglob("*_test.sh")
}
required.update(
    {
        "tests/docs/documentation_acceptance.sh",
        "tests/release/annotated_tag_identity.sh",
        "tests/release/workflow_policy.sh",
    }
)
missing = sorted(required - registered)
assert not missing, f"unregistered repository gates: {missing}"

# The runner must preserve manifest order and emit one receipt entry per gate.
fixture = root / "target/repository-gate-manifest-test.json"
subprocess.run(
    [
        sys.executable,
        "tests/ci/run_repository_gates.py",
        "--repo",
        str(root),
        "--manifest",
        str(manifest_path),
        "--report",
        str(fixture),
        "--list-only",
    ],
    cwd=root,
    check=True,
)

missing_manifest = root / "target/missing-gate-manifest.json"
missing_report = root / "target/missing-gate-report.json"
missing_manifest.write_text(
    json.dumps(
        {"schemaVersion": 1, "gates": [{"category": "fixture", "command": ["tests/missing-gate"], "id": "fixture"}]}
    ),
    encoding="utf-8",
)
missing_run = subprocess.run(
    [
        sys.executable,
        "tests/ci/run_repository_gates.py",
        "--repo",
        str(root),
        "--manifest",
        str(missing_manifest),
        "--report",
        str(missing_report),
    ],
    cwd=root,
    check=False,
    capture_output=True,
    text=True,
)
assert missing_run.returncode == 1
assert "Traceback" not in missing_run.stderr
missing_receipt = json.loads(missing_report.read_text(encoding="utf-8"))
assert missing_receipt["state"] == "failed"
assert missing_receipt["gates"][0]["state"] == "failed"
assert "launchError" in missing_receipt["gates"][0]
report = json.loads(fixture.read_text(encoding="utf-8"))
assert [gate["id"] for gate in report["gates"]] == ids
assert report["state"] == "listed"

# Git does not require every existing shell gate to carry an executable bit.
nonexec = root / "target/non-executable-gate.sh"
nonexec.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
nonexec.chmod(0o644)
nonexec_manifest = root / "target/non-executable-gate-manifest.json"
nonexec_manifest.write_text(
    json.dumps(
        {"schemaVersion": 1, "gates": [{"category": "fixture", "command": [str(nonexec)], "id": "fixture"}]}
    ),
    encoding="utf-8",
)
subprocess.run(
    [
        sys.executable,
        "tests/ci/run_repository_gates.py",
        "--repo",
        str(root),
        "--manifest",
        str(nonexec_manifest),
        "--report",
        str(root / "target/non-executable-gate-report.json"),
    ],
    cwd=root,
    check=True,
)
print("repository gate manifest regression passed")
