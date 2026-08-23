#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
gate="$root/tests/ci/governance_integrity_gate.py"
fixtures="$root/tests/ci/fixtures/governance-integrity"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/governance-integrity.XXXXXX")
trap 'rm -rf "$tmp"' EXIT

build_fixture() {
  local spec=$1
  local target=$2
  python3 "$fixtures/build_fixture.py" --spec "$spec" --output "$target"
}

run_case() {
  local name=$1
  local expected_code=$2
  local expected_finding=$3
  local repo="$tmp/$name"
  local report="$tmp/$name-report.json"
  build_fixture "$fixtures/$name.json" "$repo"
  set +e
  python3 "$gate" --repo "$repo" --report "$report" >/dev/null
  local actual_code=$?
  set -e
  [[ "$actual_code" -eq "$expected_code" ]] || {
    printf 'governance fixture %s: expected exit %s, got %s\n' "$name" "$expected_code" "$actual_code" >&2
    exit 1
  }
  python3 - "$report" "$expected_finding" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
code = sys.argv[2]
findings = [finding["code"] for finding in report["findings"]]
if code != "none" and code not in findings:
    raise SystemExit(f"expected {code}, got {findings}")
if report["findings"] != sorted(
    report["findings"], key=lambda item: (item["workItemId"], item["code"], item["path"])
):
    raise SystemExit("findings are not deterministic")
PY
}

run_case valid 0 none
run_case missing-work-item 1 missing_work_item
run_case missing-evidence 1 missing_evidence
run_case missing-close 1 missing_terminal_decision
run_case historical-exemption 0 none
run_case unknown-issue 1 unknown_problem
run_case ambiguous-current 1 ambiguous_short_id
run_case invalid-outcome 1 invalid_outcome
run_case archive-timestamp-current 0 none
run_case awaiting-merge-close 0 none
run_case merged-finalize-not-terminal 1 missing_terminal_decision
run_case invalid-premerge-finalize 1 invalid_premerge_finalize
run_case retained-premerge-finalize 1 invalid_premerge_finalize
run_case foreign-premerge-finalize 1 invalid_premerge_finalize
run_case spoofed-base-premerge-finalize 1 invalid_premerge_finalize

# The same repository snapshot must produce a byte-identical report.
cp "$tmp/valid-report.json" "$tmp/valid-report-first.json"
python3 "$gate" --repo "$tmp/valid" --report "$tmp/valid-report.json" >/dev/null
cmp "$tmp/valid-report-first.json" "$tmp/valid-report.json"

python3 - "$tmp/archive-timestamp-current-report.json" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
classification = {
    item["workItemId"]: item["classification"] for item in report["inventory"]
}
assert classification["WI-901-corrective-after-baseline"] == "current_release_cycle", classification
PY

python3 - "$tmp/awaiting-merge-close-report.json" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
item = next(
    item
    for item in report["inventory"]
    if item["workItemId"] == "WI-901-corrective-after-baseline"
)
assert item["lifecycleState"] == "awaiting_merge_close", item
assert item["decisionPath"].endswith(".finalize.json"), item
PY

python3 - "$tmp/historical-exemption-report.json" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
assert report["findings"] == [], report["findings"]
warnings = [item for item in report["legacyWarnings"] if item["code"] == "ambiguous_short_id"]
assert len(warnings) == 2, warnings
assert {item["severity"] for item in warnings} == {"historical"}, warnings
PY

# A new current Work Item must be discovered without changing an ID list.
python3 - "$fixtures/valid.json" "$tmp/dynamic.json" <<'PY'
import json
import sys

spec = json.load(open(sys.argv[1], encoding="utf-8"))
spec["workItems"].append({"id": "WI-999-release-v9-9-9-extra", "classification": "current_release"})
json.dump(spec, open(sys.argv[2], "w", encoding="utf-8"), indent=2, sort_keys=True)
PY
build_fixture "$tmp/dynamic.json" "$tmp/dynamic"
python3 "$gate" --repo "$tmp/dynamic" --report "$tmp/dynamic-report.json" >/dev/null
python3 - "$tmp/dynamic-report.json" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
ids = [item["workItemId"] for item in report["inventory"]]
assert "WI-999-release-v9-9-9-extra" in ids, ids
PY

printf 'governance integrity gate regression passed\n'
