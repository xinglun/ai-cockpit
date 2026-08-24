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
  if [[ "$name" == release-tag-* ]]; then
    GITHUB_EVENT_NAME=push \
    GITHUB_REF=refs/tags/v9.9.9 \
      GITHUB_REF_NAME=v9.9.9 \
    GITHUB_SHA="$(git -C "$repo" rev-parse HEAD)" \
      python3 "$gate" --repo "$repo" --report "$report" >/dev/null
  else
    # A release workflow exports tag context globally.  Ordinary fixtures
    # must explicitly clear it so their expected lifecycle is independent of
    # the event that launched this regression script.
    env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
      -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
      python3 "$gate" --repo "$repo" --report "$report" >/dev/null
  fi
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
run_case stale-merged-awaiting-close 1 stale_awaiting_merge_close
run_case release-tag-awaiting-close 0 none
run_case release-tag-non-ancestor 1 invalid_premerge_finalize
run_case merged-finalize-not-terminal 1 missing_terminal_decision
run_case invalid-premerge-finalize 1 invalid_premerge_finalize
run_case retained-premerge-finalize 1 invalid_premerge_finalize
run_case foreign-premerge-finalize 1 invalid_premerge_finalize
run_case spoofed-base-premerge-finalize 1 invalid_premerge_finalize
run_case superseded-recovery 0 none
run_case invalid-recovery 1 invalid_terminal_decision

# A detached pull-request merge checkout may not retain origin/HEAD or event
# base-ref metadata. The immutable Contract resource context is the narrow
# fallback; strict PR identity checks must still reject an externally known
# base-branch mismatch.
fallback_repo="$tmp/detached-default-branch"
fallback_report="$tmp/detached-default-branch-report.json"
build_fixture "$fixtures/awaiting-merge-close.json" "$fallback_repo"
git -C "$fallback_repo" symbolic-ref --delete refs/remotes/origin/HEAD
git -C "$fallback_repo" checkout --detach -q
env -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF -u GITHUB_REF \
  -u GITHUB_REF_NAME -u GITHUB_SHA \
  GITHUB_EVENT_NAME=pull_request \
  python3 "$gate" --repo "$fallback_repo" --report "$fallback_report" >/dev/null
python3 - "$fallback_report" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
assert report["state"] == "passed", report
item = next(
    item
    for item in report["inventory"]
    if item["workItemId"] == "WI-901-corrective-after-baseline"
)
assert item["lifecycleState"] == "awaiting_merge_close", item
PY

mismatch_repo="$tmp/detached-default-branch-mismatch"
mismatch_report="$tmp/detached-default-branch-mismatch-report.json"
build_fixture "$fixtures/spoofed-base-premerge-finalize.json" "$mismatch_repo"
git -C "$mismatch_repo" symbolic-ref --delete refs/remotes/origin/HEAD
git -C "$mismatch_repo" checkout --detach -q
set +e
env -u GITHUB_EVENT_PATH -u GITHUB_REF -u GITHUB_REF_NAME -u GITHUB_SHA \
  GITHUB_EVENT_NAME=pull_request GITHUB_BASE_REF=main \
  python3 "$gate" --repo "$mismatch_repo" --report "$mismatch_report" >/dev/null
mismatch_code=$?
set -e
[[ "$mismatch_code" -eq 1 ]] || {
  printf 'detached default-branch mismatch: expected exit 1, got %s\n' "$mismatch_code" >&2
  exit 1
}
python3 - "$mismatch_report" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
assert any(
    finding["workItemId"] == "WI-901-corrective-after-baseline"
    and finding["code"] == "invalid_premerge_finalize"
    for finding in report["findings"]
), report["findings"]
PY

# The same repository snapshot must produce a byte-identical report.
cp "$tmp/valid-report.json" "$tmp/valid-report-first.json"
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
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

python3 - "$tmp/stale-merged-awaiting-close-report.json" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
item = next(
    item
    for item in report["inventory"]
    if item["workItemId"] == "WI-901-release-v9-9-9"
)
assert item["lifecycleState"] == "stale_awaiting_merge_close", item
assert any(
    finding["workItemId"] == "WI-901-release-v9-9-9"
    and finding["code"] == "stale_awaiting_merge_close"
    for finding in report["findings"]
), report["findings"]
PY

python3 - "$tmp/release-tag-awaiting-close-report.json" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
item = next(
    item
    for item in report["inventory"]
    if item["workItemId"] == "WI-901-release-v9-9-9"
)
assert item["lifecycleState"] == "awaiting_merge_close", item
assert item["decisionPath"].endswith(".finalize.json"), item
PY

python3 - "$tmp/superseded-recovery-report.json" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
item = next(
    item
    for item in report["inventory"]
    if item["workItemId"] == "WI-902-recovered-predecessor"
)
assert item["lifecycleState"] == "recovered", item
assert item["decisionPath"] == ".ai/decisions/WI-902-recovered-predecessor.recovery.json", item
assert not any(
    finding["workItemId"] == "WI-902-recovered-predecessor"
    and finding["code"] == "invalid_outcome"
    for finding in report["findings"]
), report["findings"]
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
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
  python3 "$gate" --repo "$tmp/dynamic" --report "$tmp/dynamic-report.json" >/dev/null
python3 - "$tmp/dynamic-report.json" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
ids = [item["workItemId"] for item in report["inventory"]]
assert "WI-999-release-v9-9-9-extra" in ids, ids
PY

printf 'governance integrity gate regression passed\n'
