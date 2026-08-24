#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
gate="$root/tests/ci/governance_integrity_gate.py"
fixture_builder="$root/tests/ci/fixtures/governance-integrity/build_fixture.py"
fixture_spec="$root/tests/ci/fixtures/governance-integrity/awaiting-merge-close.json"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/pending-parity-registry.XXXXXX")
trap 'rm -rf "$tmp"' EXIT

repo="$tmp/valid-pending"
report="$tmp/valid-pending-report.json"
python3 "$fixture_builder" --spec "$fixture_spec" --output "$repo"

python3 - "$repo" <<'PY'
import json
import subprocess
import sys
from pathlib import Path

repo = Path(sys.argv[1])
work_item = "WI-901-corrective-after-baseline"
short_id = "WI-901"
parity_paths = (
    "docs/reference/reference-parity.md",
    "docs/reference/reference-parity.zh-CN.md",
    "docs/reference/reference-parity.ja.md",
)
expected_rows = []
for relative in parity_paths:
    path = repo / relative
    lines = path.read_text(encoding="utf-8").splitlines()
    matching = [line for line in lines if line.startswith(f"| {short_id} ")]
    assert len(matching) == 1, (relative, matching)
    pending_row = matching[0].replace("| 已实现 |", "| 进行中 |").replace(
        "| Implemented |", "| In progress |"
    )
    expected_rows.append({"path": relative, "row": pending_row})
    path.write_text(
        "\n".join(line for line in lines if line != matching[0]) + "\n",
        encoding="utf-8",
    )

subprocess.run(["git", "-C", str(repo), "add", "docs/reference"], check=True)
subprocess.run(
    ["git", "-C", str(repo), "commit", "-qm", "prepare missing parity"],
    check=True,
)
registered_head = subprocess.check_output(
    ["git", "-C", str(repo), "rev-parse", "HEAD"], text=True
).strip()
project = json.loads((repo / ".ai/project.json").read_text(encoding="utf-8"))
finalize_path = repo / ".ai/decisions" / f"{work_item}.finalize.json"
finalize = json.loads(finalize_path.read_text(encoding="utf-8"))
finalize["branch"]["headRevision"] = registered_head
finalize["pullRequest"]["headRevision"] = registered_head
finalize["worktree"]["headRevision"] = registered_head
finalize_path.write_text(
    json.dumps(finalize, indent=2, sort_keys=True) + "\n", encoding="utf-8"
)
subprocess.run(["git", "-C", str(repo), "add", str(finalize_path)], check=True)
subprocess.run(
    ["git", "-C", str(repo), "commit", "-qm", "append canonical finalization"],
    check=True,
)
registry_base_revision = subprocess.check_output(
    ["git", "-C", str(repo), "rev-parse", "HEAD"], text=True
).strip()
registry = {
    "schemaVersion": 1,
    "entries": [
        {
            "baseRevision": finalize["pullRequest"]["baseRevision"],
            "createdAt": "2026-03-02T00:00:00Z",
            "expectedRecords": {
                "contract": f".ai/work-items/archive/{work_item}.contract.json",
                "finalize": f".ai/decisions/{work_item}.finalize.json",
                "verification": f".ai/evidence/{work_item}.verification.json",
            },
            "headRevision": registered_head,
            "parityRows": expected_rows,
            "provider": "github",
            "pullRequest": {
                "number": finalize["pullRequest"]["number"],
                "url": finalize["pullRequest"]["url"],
            },
            "repositoryId": project["repositoryId"],
            "registryBaseRevision": registry_base_revision,
            "state": "in_progress",
            "workItemId": work_item,
        }
    ],
}
registry_path = repo / "docs/reference/pending-parity-registry.json"
registry_path.write_text(
    json.dumps(registry, indent=2, sort_keys=True) + "\n", encoding="utf-8"
)
subprocess.run(["git", "-C", str(repo), "add", str(registry_path)], check=True)
subprocess.run(
    ["git", "-C", str(repo), "commit", "-qm", "append pending registry"],
    check=True,
)
PY

env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
  python3 "$gate" --repo "$repo" --report "$report" >/dev/null

python3 - "$report" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
assert report["state"] == "passed", report["findings"]
item = next(
    item
    for item in report["inventory"]
    if item["workItemId"] == "WI-901-corrective-after-baseline"
)
assert item["lifecycleState"] == "pending_parity_registration", item
assert item["pendingParityRegistryPath"] == (
    "docs/reference/pending-parity-registry.json"
), item
PY

mutate_case() {
  local case_name=$1
  local case_repo=$2
  python3 - "$case_repo" "$case_name" <<'PY'
import json
import os
import subprocess
import sys
from pathlib import Path

repo = Path(sys.argv[1])
case = sys.argv[2]
registry_path = repo / "docs/reference/pending-parity-registry.json"
registry = json.loads(registry_path.read_text(encoding="utf-8"))
entry = registry["entries"][0]
work_item = entry["workItemId"]

if case == "foreign-repository":
    entry["repositoryId"] = "sha256:" + "f" * 64
elif case == "head-mismatch":
    entry["headRevision"] = subprocess.check_output(
        ["git", "-C", str(repo), "rev-parse", f"{entry['headRevision']}^"],
        text=True,
    ).strip()
elif case == "base-mismatch":
    entry["baseRevision"] = "f" * 40
elif case == "pull-request-mismatch":
    entry["pullRequest"]["number"] += 1
    entry["pullRequest"]["url"] = (
        "https://github.com/example/fixture/pull/1000"
    )
elif case == "unsafe-record-path":
    entry["expectedRecords"]["verification"] = "../foreign.json"
elif case == "parity-row-mismatch":
    entry["parityRows"][0]["row"] = entry["parityRows"][0]["row"].replace(
        "In progress", "Implemented"
    )
elif case == "invalid-created-at":
    entry["createdAt"] = "not-an-rfc3339-timestamp"
elif case == "unknown-field":
    entry["unexpected"] = True
elif case == "duplicate-entry":
    registry["entries"].append(dict(entry))
elif case == "duplicate-key":
    encoded = json.dumps(registry, indent=2, sort_keys=True) + "\n"
    registry_path.write_text(
        encoded.rstrip().removesuffix("}")
        + ',\n  "schemaVersion": 1\n}\n',
        encoding="utf-8",
    )
elif case == "missing-record":
    (repo / entry["expectedRecords"]["verification"]).unlink()
elif case == "symlink-record":
    evidence = repo / entry["expectedRecords"]["verification"]
    evidence.unlink()
    evidence.symlink_to(repo / ".ai/project.json")
elif case == "foreign-contract":
    contract = repo / entry["expectedRecords"]["contract"]
    value = json.loads(contract.read_text(encoding="utf-8"))
    value["repositoryId"] = "sha256:" + "f" * 64
    contract.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
elif case == "runtime-mismatch":
    evidence = repo / entry["expectedRecords"]["verification"]
    value = json.loads(evidence.read_text(encoding="utf-8"))
    value["runtimeDigest"] = "sha256:" + "f" * 64
    evidence.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
elif case == "unrelated-append":
    (repo / "unrelated.txt").write_text("not governance\n", encoding="utf-8")
elif case == "partial-parity-row":
    parity = entry["parityRows"][0]
    path = repo / parity["path"]
    path.write_text(path.read_text(encoding="utf-8") + parity["row"] + "\n", encoding="utf-8")
elif case == "default-branch":
    subprocess.run(["git", "-C", str(repo), "checkout", "-q", "main"], check=True)
    subprocess.run(
        [
            "git",
            "-C",
            str(repo),
            "merge",
            "--no-ff",
            "-qm",
            "merge pending fixture",
            "codex/fixture",
        ],
        check=True,
    )
elif case == "symlink-registry":
    registry_path.unlink()
    registry_path.symlink_to("reference-parity.md")
elif case == "broken-symlink-registry":
    registry_path.unlink()
    registry_path.symlink_to("missing-pending-registry.json")
else:
    raise AssertionError(f"unknown fixture mutation: {case}")

if case not in {
    "broken-symlink-registry",
    "duplicate-key",
    "default-branch",
    "symlink-registry",
}:
    registry_path.write_text(
        json.dumps(registry, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
if case != "default-branch":
    subprocess.run(["git", "-C", str(repo), "add", "-A"], check=True)
    subprocess.run(
        ["git", "-C", str(repo), "commit", "-qm", f"mutate {case}"],
        check=True,
    )
PY
}

run_invalid_case() {
  local case_name=$1
  local expected_finding=$2
  local case_repo="$tmp/$case_name"
  local case_report="$tmp/$case_name-report.json"
  cp -R "$repo" "$case_repo"
  mutate_case "$case_name" "$case_repo"
  set +e
  env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
    -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
    python3 "$gate" --repo "$case_repo" --report "$case_report" >/dev/null
  local wi244_case_exit=$?
  set -e
  [[ "$wi244_case_exit" -eq 1 ]] || {
    printf '%s: expected exit 1, got %s\n' "$case_name" "$wi244_case_exit" >&2
    exit 1
  }
  python3 - "$case_report" "$expected_finding" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
expected = sys.argv[2]
codes = [finding["code"] for finding in report["findings"]]
assert expected in codes, (expected, codes)
assert report["findings"] == sorted(
    report["findings"],
    key=lambda item: (item["workItemId"], item["code"], item["path"]),
), report["findings"]
PY
}

run_invalid_case foreign-repository invalid_pending_parity_registration
run_invalid_case head-mismatch invalid_pending_parity_registration
run_invalid_case base-mismatch invalid_pending_parity_registration
run_invalid_case pull-request-mismatch invalid_pending_parity_registration
run_invalid_case unsafe-record-path invalid_pending_parity_registration
run_invalid_case parity-row-mismatch invalid_pending_parity_registration
run_invalid_case invalid-created-at invalid_pending_parity_registration
run_invalid_case unknown-field invalid_pending_parity_registry
run_invalid_case duplicate-entry invalid_pending_parity_registry
run_invalid_case duplicate-key invalid_pending_parity_registry
run_invalid_case missing-record invalid_pending_parity_registration
run_invalid_case symlink-record invalid_pending_parity_registration
run_invalid_case symlink-registry invalid_pending_parity_registry
run_invalid_case broken-symlink-registry invalid_pending_parity_registry
run_invalid_case foreign-contract invalid_pending_parity_registration
run_invalid_case runtime-mismatch invalid_pending_parity_registration
run_invalid_case unrelated-append invalid_pending_parity_registration
run_invalid_case partial-parity-row stale_pending_parity_registration
run_invalid_case default-branch stale_pending_parity_registration

printf 'pending parity registry regression passed\n'
