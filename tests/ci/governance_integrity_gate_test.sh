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
  test -f "$repo/docs/reference/pending-parity-registry.json"
  python3 - "$repo/docs/reference/pending-parity-registry.json" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
value = json.loads(path.read_text(encoding="utf-8"))
assert path.is_file() and not path.is_symlink(), path
assert value == {"entries": [], "schemaVersion": 1}, value
PY
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

# ``confirmed`` is an explicit positive Runtime decision token equivalent to
# ``approved`` for terminal promotion; arbitrary/rejected decisions remain
# non-green.
build_fixture "$fixtures/valid.json" "$tmp/confirmed-decision"
python3 - "$tmp/confirmed-decision" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
for path in root.glob(".ai/decisions/*.close.json"):
    value = json.loads(path.read_text(encoding="utf-8"))
    value["humanDecision"] = "confirmed"
    value["structuredDecision"]["decision"] = "confirmed"
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
PY
python3 "$gate" --repo "$tmp/confirmed-decision" --report "$tmp/confirmed-decision-report.json" >/dev/null
python3 - "$tmp/confirmed-decision-report.json" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
assert report["findings"] == [], report["findings"]
PY
printf 'governance confirmed-decision regression passed\n'

run_case missing-work-item 1 missing_work_item
run_case missing-evidence 1 missing_evidence
run_case missing-close 1 missing_terminal_decision
run_case historical-exemption 0 none
run_case unknown-issue 1 unknown_problem
run_case ambiguous-current 1 ambiguous_short_id
run_case invalid-outcome 1 invalid_outcome
run_case archive-timestamp-current 0 none
run_case awaiting-merge-close 0 none
run_case runtime-audit-reason 0 none
run_case empty-audit-reason 1 invalid_premerge_finalize
run_case whitespace-audit-reason 1 invalid_premerge_finalize
run_case stale-merged-awaiting-close 1 stale_awaiting_merge_close
run_case release-tag-awaiting-close 0 none
run_case release-tag-non-ancestor 1 invalid_premerge_finalize
run_case merged-finalize-not-terminal 1 missing_terminal_decision
run_case invalid-premerge-finalize 1 invalid_premerge_finalize
run_case retained-premerge-finalize 1 invalid_premerge_finalize
run_case foreign-premerge-finalize 1 invalid_premerge_finalize
run_case spoofed-base-premerge-finalize 1 invalid_premerge_finalize

# An immutable retry is not a terminal decision.  Without a successor (or a
# normal finalize/close chain), the gate must keep the predecessor open and
# fail closed rather than treating the retry receipt as a completed delivery.
build_fixture "$fixtures/valid.json" "$tmp/orphaned-retry"
python3 - "$tmp/orphaned-retry" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-900-release-v9-9-9"
decisions = root / ".ai/decisions"
(decisions / f"{work_item}.close.json").unlink()
project = json.loads((root / ".ai/project.json").read_text(encoding="utf-8"))
(decisions / f"{work_item}.recovery.json").write_text(
    json.dumps(
        {
            "schemaVersion": 1,
            "workItemId": work_item,
            "predecessorWorkItemId": work_item,
            "successorWorkItemId": None,
            "decision": "retry",
            "repositoryId": project["repositoryId"],
            "predecessorContractDigest": "sha256:" + "c" * 64,
            "predecessorSummaryDigest": "sha256:" + "d" * 64,
            "predecessorOutcomeDigest": "sha256:" + "e" * 64,
            "reason": "Preserve an immutable failed delivery while awaiting an explicit successor.",
            "evidenceRefs": [f".ai/evidence/{work_item}.verification.json"],
        },
        indent=2,
    )
    + "\n",
    encoding="utf-8",
)
for name in ("reference-parity.md", "reference-parity.zh-CN.md", "reference-parity.ja.md"):
    path = root / "docs/reference" / name
    text = path.read_text(encoding="utf-8")
    text = text.replace(
        f"`.ai/decisions/{work_item}.close.json`",
        f"`.ai/decisions/{work_item}.recovery.json`",
    )
    path.write_text(text, encoding="utf-8")
PY
set +e
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
  python3 "$gate" --repo "$tmp/orphaned-retry" \
  --report "$tmp/orphaned-retry-report.json" >/dev/null
orphaned_retry_code=$?
set -e
[[ "$orphaned_retry_code" -eq 1 ]] || {
  printf 'orphaned retry: expected exit 1, got %s\n' "$orphaned_retry_code" >&2
  exit 1
}
python3 - "$tmp/orphaned-retry-report.json" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
findings = report["findings"]
assert any(
    finding["workItemId"] == "WI-900-release-v9-9-9"
    and finding["code"] == "missing_terminal_decision"
    for finding in findings
), findings
item = next(
    item
    for item in report["inventory"]
    if item["workItemId"] == "WI-900-release-v9-9-9"
)
assert item["lifecycleState"] == "closure_missing", item
assert not any(
    finding["workItemId"] == "WI-900-release-v9-9-9"
    and finding["code"] == "invalid_terminal_decision"
    for finding in findings
), findings
PY
printf 'governance orphaned-retry regression passed\n'

# Runtime recovery is append-only. A canonical retry may coexist with a
# digest-suffixed successor/supersession receipt; the latest valid terminal
# recovery must be selected and bound into all parity rows.
build_fixture "$fixtures/valid.json" "$tmp/recovery-suffixed"
python3 - "$tmp/recovery-suffixed" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-900-release-v9-9-9"
decisions = root / ".ai/decisions"
decisions.mkdir(parents=True, exist_ok=True)
(decisions / f"{work_item}.close.json").unlink()
common = {
    "schemaVersion": 1,
    "decisionId": "work-item-recovery",
    "workItemId": work_item,
    "repositoryId": "sha256:" + "b" * 64,
    "predecessorWorkItemId": work_item,
    "runtimeVersion": "0.2.23",
    "runtimeDigest": "sha256:" + "a" * 64,
    "actor": "human:fixture",
    "authoritySource": "fixture",
    "reason": "preserve immutable predecessor history",
    "evidenceRefs": [f".ai/evidence/{work_item}.verification.json"],
    "policyRefs": [],
    "decidedAt": "2026-03-01T00:00:00Z",
    "resumeCondition": "continue through the successor",
}
retry = dict(common, decision="retry")
retry["predecessorContractDigest"] = "sha256:" + "c" * 64
retry["predecessorSummaryDigest"] = "sha256:" + "d" * 64
(decisions / f"{work_item}.recovery.json").write_text(
    json.dumps(retry, indent=2) + "\n", encoding="utf-8"
)
supersede = dict(common, decision="supersede", successorWorkItemId="WI-901-corrective-after-baseline")
supersede["predecessorContractDigest"] = retry["predecessorContractDigest"]
supersede["predecessorSummaryDigest"] = retry["predecessorSummaryDigest"]
supersede["decidedAt"] = "2026-03-02T00:00:00Z"
suffix = "a" * 64
(decisions / f"{work_item}.recovery.{suffix}.json").write_text(
    json.dumps(supersede, indent=2) + "\n", encoding="utf-8"
)
for name in ("reference-parity.md", "reference-parity.zh-CN.md", "reference-parity.ja.md"):
    path = root / "docs/reference" / name
    text = path.read_text(encoding="utf-8")
    text = text.replace(
        f"`.ai/decisions/{work_item}.close.json`",
        f"`.ai/decisions/{work_item}.recovery.{suffix}.json`",
    )
    path.write_text(text, encoding="utf-8")
PY
python3 "$gate" --repo "$tmp/recovery-suffixed" --report "$tmp/recovery-suffixed-report.json" >/dev/null
python3 - "$tmp/recovery-suffixed-report.json" <<'PY'
import json
import sys
report = json.load(open(sys.argv[1], encoding="utf-8"))
assert report["findings"] == [], report["findings"]
item = next(item for item in report["inventory"] if item["workItemId"] == "WI-900-release-v9-9-9")
assert item["decisionPath"].endswith(".recovery." + "a" * 64 + ".json"), item
assert item["lifecycleState"] == "recovered", item
PY
printf 'governance recovery suffix regression passed\n'

# A successful retry remains immutable history and must not turn an otherwise
# normal archived/finalized item into a recovered predecessor. The static gate
# must continue to project the real finalization boundary.
build_fixture "$fixtures/awaiting-merge-close.json" "$tmp/successful-retry-history"
python3 - "$tmp/successful-retry-history" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-901-corrective-after-baseline"
project = json.loads((root / ".ai/project.json").read_text(encoding="utf-8"))
summary_path = root / ".ai/work-items/archive" / f"{work_item}.summary.json"
summary = json.loads(summary_path.read_text(encoding="utf-8"))
summary.update(
    {
        "outcomeState": "blocked",
        "failedGate": "finish.lifecycle",
        "recoveryCondition": "Restore the required lifecycle state before retrying.",
    }
)
summary_path.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
(root / ".ai/decisions" / f"{work_item}.recovery.json").write_text(
    json.dumps(
        {
            "schemaVersion": 1,
            "workItemId": work_item,
            "predecessorWorkItemId": work_item,
            "successorWorkItemId": None,
            "decision": "retry",
            "repositoryId": project["repositoryId"],
            "reason": "Retry a failed lifecycle transition without rewriting history.",
            "evidenceRefs": [f".ai/evidence/{work_item}.verification.json"],
        },
        indent=2,
    )
    + "\n",
    encoding="utf-8",
)
PY
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF python3 "$gate" --repo "$tmp/successful-retry-history" --report "$tmp/successful-retry-history-report.json" >/dev/null
python3 - "$tmp/successful-retry-history-report.json" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
assert report["findings"] == [], report["findings"]
item = next(
    item
    for item in report["inventory"]
    if item["workItemId"] == "WI-901-corrective-after-baseline"
)
assert item["lifecycleState"] == "awaiting_merge_close", item
assert item["decisionPath"].endswith(".finalize.json"), item
PY
printf 'governance successful-retry history regression passed\n'

# Runtime also consumes a retry when its predecessor Contract/Summary binding
# is stale after fresh verification. The archived Summary is then the normal
# finish_ready projection (without the transient blocked fields), so the gate
# must use the immutable predecessor digests rather than requiring a marker
# that no longer exists.
build_fixture "$fixtures/awaiting-merge-close.json" "$tmp/stale-retry-after-fresh-verify"
python3 - "$tmp/stale-retry-after-fresh-verify" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-901-corrective-after-baseline"
project = json.loads((root / ".ai/project.json").read_text(encoding="utf-8"))
(root / ".ai/decisions" / f"{work_item}.recovery.json").write_text(
    json.dumps(
        {
            "schemaVersion": 1,
            "workItemId": work_item,
            "predecessorWorkItemId": work_item,
            "successorWorkItemId": None,
            "decision": "retry",
            "repositoryId": project["repositoryId"],
            "predecessorContractDigest": "sha256:" + "c" * 64,
            "predecessorSummaryDigest": "sha256:" + "d" * 64,
            "reason": "A fresh verification advanced the archived bindings.",
            "evidenceRefs": [f".ai/evidence/{work_item}.verification.json"],
        },
        indent=2,
    )
    + "\n",
    encoding="utf-8",
)
PY
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF python3 "$gate" --repo "$tmp/stale-retry-after-fresh-verify" --report "$tmp/stale-retry-after-fresh-verify-report.json" >/dev/null
python3 - "$tmp/stale-retry-after-fresh-verify-report.json" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
assert report["findings"] == [], report["findings"]
item = next(
    item
    for item in report["inventory"]
    if item["workItemId"] == "WI-901-corrective-after-baseline"
)
assert item["lifecycleState"] == "awaiting_merge_close", item
assert item["decisionPath"].endswith(".finalize.json"), item
PY
printf 'governance stale-retry binding regression passed\n'

# The same consumed retry remains historical after a valid close decision is
# present.  Closing the item must not turn the historical parity warning into
# a blocking current-cycle error.
python3 - "$tmp/successful-retry-history" "$tmp/valid" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
source = Path(sys.argv[2])
work_item = "WI-901-corrective-after-baseline"
close = json.loads(
    (source / ".ai/decisions/WI-900-release-v9-9-9.close.json").read_text(
        encoding="utf-8"
    )
)
close["workItemId"] = work_item
close["structuredDecision"]["evidenceRefs"] = [
    f".ai/evidence/{work_item}.verification.json"
]
(root / ".ai/decisions" / f"{work_item}.close.json").write_text(
    json.dumps(close, indent=2) + "\n", encoding="utf-8"
)
for name in ("reference-parity.md", "reference-parity.zh-CN.md", "reference-parity.ja.md"):
    path = root / "docs/reference" / name
    text = path.read_text(encoding="utf-8")
    text = text.replace(
        f"`.ai/decisions/{work_item}.finalize.json`",
        f"`.ai/decisions/{work_item}.finalize.json`; `.ai/decisions/{work_item}.close.json`",
    )
    path.write_text(text, encoding="utf-8")
PY
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF python3 "$gate" --repo "$tmp/successful-retry-history" --report "$tmp/successful-retry-history-closed-report.json" >/dev/null
python3 - "$tmp/successful-retry-history-closed-report.json" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
assert report["findings"] == [], report["findings"]
item = next(
    item
    for item in report["inventory"]
    if item["workItemId"] == "WI-901-corrective-after-baseline"
)
assert item["lifecycleState"] == "closed", item
PY
printf 'governance successful-retry closed-history regression passed\n'

# A predecessor may have a structurally valid close whose human decision is
# explicitly superseded.  The recovery receipt must still own the inventory
# projection so the parity row cannot claim a green implementation.
build_fixture "$fixtures/valid.json" "$tmp/superseded-structured-close"
python3 - "$tmp/superseded-structured-close" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
work_item = "WI-900-release-v9-9-9"
repository_id = "sha256:" + "b" * 64
decisions = root / ".ai/decisions"
close_path = decisions / f"{work_item}.close.json"
close = json.loads(close_path.read_text(encoding="utf-8"))
close["humanDecision"] = "superseded"
close["structuredDecision"]["decision"] = "superseded"
close_path.write_text(json.dumps(close, indent=2) + "\n", encoding="utf-8")
recovery_path = decisions / f"{work_item}.recovery.json"
recovery_path.write_text(
    json.dumps(
        {
            "schemaVersion": 1,
            "workItemId": work_item,
            "predecessorWorkItemId": work_item,
            "successorWorkItemId": "WI-901-corrective-after-baseline",
            "decision": "supersede",
            "repositoryId": repository_id,
            "reason": "Preserve the predecessor as immutable history.",
            "evidenceRefs": [f".ai/evidence/{work_item}.verification.json"],
        },
        indent=2,
    )
    + "\n",
    encoding="utf-8",
)
for name in ("reference-parity.md", "reference-parity.zh-CN.md", "reference-parity.ja.md"):
    path = root / "docs/reference" / name
    text = path.read_text(encoding="utf-8")
    text = text.replace(
        f"`.ai/decisions/{work_item}.close.json`",
        f"`.ai/decisions/{work_item}.recovery.json`",
    )
    text = text.replace("| Implemented |", "| Recovered |", 1)
    text = text.replace("| 已实现 |", "| 已恢复 |", 1)
    path.write_text(text, encoding="utf-8")
PY
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
  python3 "$gate" --repo "$tmp/superseded-structured-close" \
  --report "$tmp/superseded-structured-close-report.json" >/dev/null
python3 - "$tmp/superseded-structured-close-report.json" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
assert report["findings"] == [], report["findings"]
item = next(
    item
    for item in report["inventory"]
    if item["workItemId"] == "WI-900-release-v9-9-9"
)
assert item["lifecycleState"] == "recovered", item
assert item["decisionPath"].endswith(".recovery.json"), item
PY
printf 'governance structured superseded-close regression passed\n'
run_case superseded-recovery 0 none
run_case invalid-recovery 1 invalid_terminal_decision

# Provider receipts may use an unambiguous abbreviated Git revision. Resolve
# it to the exact commit object before binding the finalization identity.
short_head_repo="$tmp/short-head-finalize"
short_head_report="$tmp/short-head-finalize-report.json"
build_fixture "$fixtures/awaiting-merge-close.json" "$short_head_repo"
python3 - "$short_head_repo" <<'PY'
import json
import subprocess
import sys
from pathlib import Path

repo = Path(sys.argv[1])
path = repo / ".ai/decisions/WI-901-corrective-after-baseline.finalize.json"
value = json.loads(path.read_text(encoding="utf-8"))
full = subprocess.check_output(
    ["git", "rev-parse", "HEAD^{commit}"], cwd=repo, text=True
).strip()
short = full[:7]
value["pullRequest"]["headRevision"] = short
value["branch"]["headRevision"] = short
value["worktree"]["headRevision"] = short
path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
  python3 "$gate" --repo "$short_head_repo" --report "$short_head_report" >/dev/null
python3 - "$short_head_report" <<'PY'
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

# A finalization receipt is bound to the reviewed checkout head, not merely
# to a self-consistent head value copied into the receipt. Later code drift
# must force a fresh finalization instead of remaining accepted.
drift_repo="$tmp/post-finalization-code-drift"
drift_report="$tmp/post-finalization-code-drift-report.json"
build_fixture "$fixtures/awaiting-merge-close.json" "$drift_repo"
printf 'post-finalization drift\n' > "$drift_repo/post-finalization-drift.txt"
git -C "$drift_repo" add post-finalization-drift.txt
git -C "$drift_repo" commit -qm "post-finalization code drift"
set +e
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
  python3 "$gate" --repo "$drift_repo" --report "$drift_report" >/dev/null
drift_code=$?
set -e
[[ "$drift_code" -eq 1 ]] || {
  printf 'post-finalization drift: expected exit 1, got %s\n' "$drift_code" >&2
  exit 1
}
python3 - "$drift_report" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
assert any(
    finding["workItemId"] == "WI-901-corrective-after-baseline"
    and finding["code"] == "invalid_premerge_finalize"
    for finding in report["findings"]
), report["findings"]
PY

# An immutable predecessor may retain a non-canonical historical close while a
# valid recovery receipt supersedes it.  Recovery must be the terminal
# projection; the stale close must not re-open a closure_invalid finding.
recovered_close_repo="$tmp/recovered-with-invalid-close"
recovered_close_report="$tmp/recovered-with-invalid-close-report.json"
build_fixture "$fixtures/superseded-recovery.json" "$recovered_close_repo"
python3 - "$recovered_close_repo/.ai/decisions/WI-902-recovered-predecessor.close.json" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
path.write_text(
    json.dumps(
        {
            "workItemId": "WI-902-recovered-predecessor",
            "repositoryId": "sha256:" + "b" * 64,
            "state": "closed",
            "decisionState": "confirmed",
            "humanDecision": "historical descriptive close",
        },
        indent=2,
    )
    + "\n",
    encoding="utf-8",
)
PY
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
  python3 "$gate" --repo "$recovered_close_repo" --report "$recovered_close_report" >/dev/null
python3 - "$recovered_close_report" <<'PY'
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
    and finding["code"] == "invalid_terminal_decision"
    for finding in report["findings"]
), report["findings"]
PY

# A retry recovery is predecessor-bound but intentionally has no successor
# Contract. The static gate must project it as recovered without requiring an
# invented successor identity.
retry_recovery_repo="$tmp/retry-recovery"
retry_recovery_report="$tmp/retry-recovery-report.json"
build_fixture "$fixtures/superseded-recovery.json" "$retry_recovery_repo"
python3 - "$retry_recovery_repo/.ai/decisions/WI-902-recovered-predecessor.recovery.json" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
value = json.loads(path.read_text(encoding="utf-8"))
value["decision"] = "retry"
value.pop("successorWorkItemId", None)
path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
  python3 "$gate" --repo "$retry_recovery_repo" --report "$retry_recovery_report" >/dev/null
python3 - "$retry_recovery_report" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
assert report["state"] == "passed", report
item = next(
    item
    for item in report["inventory"]
    if item["workItemId"] == "WI-902-recovered-predecessor"
)
assert item["lifecycleState"] == "recovered", item
PY

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

python3 - "$tmp/runtime-audit-reason-report.json" <<'PY'
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

# A Work Item that changes the parity ledger must project its own lifecycle
# row before verification.  The same exact row remains truthful while active,
# awaiting merge/close, and closed, so archive/finalization never requires a
# post-verification documentation mutation.
prearchive_repo="$tmp/prearchive-parity-projection"
prearchive_staged="$tmp/prearchive-parity-staged"
prearchive_missing_report="$tmp/prearchive-parity-missing-report.json"
prearchive_active_report="$tmp/prearchive-parity-active-report.json"
prearchive_finalize_report="$tmp/prearchive-parity-finalize-report.json"
prearchive_close_report="$tmp/prearchive-parity-close-report.json"
build_fixture "$fixtures/awaiting-merge-close.json" "$prearchive_repo"
python3 - "$prearchive_repo" "$prearchive_staged" <<'PY'
import json
import shutil
import subprocess
import sys
from pathlib import Path

repo = Path(sys.argv[1])
staged = Path(sys.argv[2])
work_item = "WI-901-corrective-after-baseline"
short_id = "WI-901"

for relative in (
    f".ai/work-items/archive/{work_item}.archive.json",
    f".ai/work-items/archive/{work_item}.contract.json",
    f".ai/work-items/archive/{work_item}.outcome.json",
    f".ai/work-items/archive/{work_item}.summary.json",
    f".ai/evidence/{work_item}.verification.json",
    f".ai/decisions/{work_item}.finalize.json",
):
    source = repo / relative
    destination = staged / relative
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.move(source, destination)

active = repo / ".ai/work-items/active"
active.mkdir(parents=True, exist_ok=True)
shutil.copy2(
    staged / ".ai/work-items/archive" / f"{work_item}.contract.json",
    active / f"{work_item}.contract.json",
)
shutil.copy2(
    staged / ".ai/work-items/archive" / f"{work_item}.summary.json",
    active / f"{work_item}.summary.json",
)
contract_path = active / f"{work_item}.contract.json"
contract = json.loads(contract_path.read_text(encoding="utf-8"))
contract["scope"] = [
    "docs/reference/reference-parity.md",
    "docs/reference/reference-parity.zh-CN.md",
    "docs/reference/reference-parity.ja.md",
]
contract["acceptanceCriteria"] = [
    "Register the Work Item in the tri-language parity ledger before archive."
]
contract_path.write_text(
    json.dumps(contract, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
)
for relative in (
    "docs/reference/reference-parity.md",
    "docs/reference/reference-parity.zh-CN.md",
    "docs/reference/reference-parity.ja.md",
):
    path = repo / relative
    path.write_text(
        "\n".join(
            line
            for line in path.read_text(encoding="utf-8").splitlines()
            if not line.startswith(f"| {short_id} ")
        )
        + "\n",
        encoding="utf-8",
    )
subprocess.run(["git", "add", "-A"], cwd=repo, check=True)
subprocess.run(
    ["git", "commit", "-qm", "prepare active parity work item"],
    cwd=repo,
    check=True,
)
PY
set +e
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
  python3 "$gate" --repo "$prearchive_repo" \
    --report "$prearchive_missing_report" >/dev/null
prearchive_missing_code=$?
set -e
[[ "$prearchive_missing_code" -eq 1 ]] || {
  printf 'prearchive parity missing: expected exit 1, got %s\n' \
    "$prearchive_missing_code" >&2
  exit 1
}
python3 - "$prearchive_missing_report" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
findings = [
    finding
    for finding in report["findings"]
    if finding["workItemId"] == "WI-901-corrective-after-baseline"
    and finding["code"] == "missing_prearchive_parity_entry"
]
assert len(findings) == 3, findings
PY

# Ordinary code Work Items do not own the parity ledger.  The light gate must
# discover that boundary from Contract/Summary intent and remain dormant; the
# same conditional check is inherited by standard and strict profiles.
nonparity_repo="$tmp/nonparity-active-work-item"
nonparity_report="$tmp/nonparity-active-work-item-report.json"
cp -R "$prearchive_repo" "$nonparity_repo"
python3 - "$nonparity_repo" <<'PY'
import json
import subprocess
import sys
from pathlib import Path

repo = Path(sys.argv[1])
work_item = "WI-901-corrective-after-baseline"
contract_path = repo / ".ai/work-items/active" / f"{work_item}.contract.json"
contract = json.loads(contract_path.read_text(encoding="utf-8"))
contract["scope"] = ["crates/example/src/lib.rs"]
contract["acceptanceCriteria"] = ["The bounded code change passes its tests."]
contract_path.write_text(
    json.dumps(contract, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
)
subprocess.run(["git", "add", str(contract_path.relative_to(repo))], cwd=repo, check=True)
subprocess.run(
    ["git", "commit", "-qm", "declare non-parity active work item"],
    cwd=repo,
    check=True,
)
PY
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
  python3 "$gate" --repo "$nonparity_repo" --report "$nonparity_report" >/dev/null
python3 - "$nonparity_report" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
assert report["state"] == "passed", report["findings"]
item = next(
    item
    for item in report["inventory"]
    if item["workItemId"] == "WI-901-corrective-after-baseline"
)
assert item["lifecycleState"] == "active_non_parity", item
PY

python3 - "$prearchive_repo" "$prearchive_staged" <<'PY'
import shutil
import subprocess
import sys
from pathlib import Path

repo = Path(sys.argv[1])
staged = Path(sys.argv[2])
work_item = "WI-901-corrective-after-baseline"
short_id = "WI-901"
contract = f".ai/work-items/archive/{work_item}.contract.json"
evidence = f".ai/evidence/{work_item}.verification.json"
finalize = f".ai/decisions/{work_item}.finalize.json"
close = f".ai/decisions/{work_item}.close.json"
statuses = {
    "docs/reference/reference-parity.md": (
        "In progress → Implemented after verified close",
        ";",
    ),
    "docs/reference/reference-parity.zh-CN.md": (
        "进行中 → 验证关闭后已实现",
        "；",
    ),
    "docs/reference/reference-parity.ja.md": (
        "In progress → verified close 後 Implemented",
        "；",
    ),
}
for relative, (status, separator) in statuses.items():
    path = repo / relative
    row = (
        f"| {short_id} — fixture | {status} | "
        f"`{contract}`{separator} `{evidence}`{separator} "
        f"`{finalize}`{separator} `{close}` |"
    )
    path.write_text(path.read_text(encoding="utf-8") + row + "\n", encoding="utf-8")
    snapshot = staged / "parity" / path.name
    snapshot.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(path, snapshot)
subprocess.run(["git", "add", "docs/reference"], cwd=repo, check=True)
subprocess.run(
    ["git", "commit", "-qm", "project lifecycle-bound parity rows"],
    cwd=repo,
    check=True,
)
PY
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
  python3 "$gate" --repo "$prearchive_repo" \
    --report "$prearchive_active_report" >/dev/null
python3 - "$prearchive_active_report" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
assert report["state"] == "passed", report["findings"]
item = next(
    item
    for item in report["inventory"]
    if item["workItemId"] == "WI-901-corrective-after-baseline"
)
assert item["lifecycleState"] == "prearchive_parity_registered", item
PY

run_prearchive_invalid_case() {
  local case_name=$1
  local expected_finding=$2
  local case_repo="$tmp/prearchive-$case_name"
  local case_report="$tmp/prearchive-$case_name-report.json"
  cp -R "$prearchive_repo" "$case_repo"
  python3 - "$case_repo" "$case_name" <<'PY'
import subprocess
import sys
from pathlib import Path

repo = Path(sys.argv[1])
case = sys.argv[2]
work_item = "WI-901-corrective-after-baseline"
short_id = "WI-901"
paths = {
    "en": repo / "docs/reference/reference-parity.md",
    "zh": repo / "docs/reference/reference-parity.zh-CN.md",
    "ja": repo / "docs/reference/reference-parity.ja.md",
}
if case == "partial-row":
    path = paths["ja"]
    path.write_text(
        "\n".join(
            line
            for line in path.read_text(encoding="utf-8").splitlines()
            if not line.startswith(f"| {short_id} ")
        )
        + "\n",
        encoding="utf-8",
    )
elif case == "terminal-status":
    path = paths["en"]
    path.write_text(
        path.read_text(encoding="utf-8").replace(
            "In progress → Implemented after verified close",
            "Implemented",
        ),
        encoding="utf-8",
    )
elif case == "foreign-path":
    path = paths["zh"]
    path.write_text(
        path.read_text(encoding="utf-8").replace(
            f".ai/decisions/{work_item}.close.json",
            ".ai/decisions/WI-999-foreign.close.json",
        ),
        encoding="utf-8",
    )
else:
    raise AssertionError(case)
subprocess.run(["git", "add", "docs/reference"], cwd=repo, check=True)
subprocess.run(["git", "commit", "-qm", f"mutate {case}"], cwd=repo, check=True)
PY
  set +e
  env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
    -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
    python3 "$gate" --repo "$case_repo" --report "$case_report" >/dev/null
  local actual_code=$?
  set -e
  [[ "$actual_code" -eq 1 ]] || {
    printf 'prearchive %s: expected exit 1, got %s\n' "$case_name" \
      "$actual_code" >&2
    exit 1
  }
  python3 - "$case_report" "$expected_finding" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
expected = sys.argv[2]
assert any(
    finding["workItemId"] == "WI-901-corrective-after-baseline"
    and finding["code"] == expected
    for finding in report["findings"]
), report["findings"]
PY
}

run_prearchive_invalid_case partial-row missing_prearchive_parity_entry
run_prearchive_invalid_case terminal-status invalid_prearchive_parity_registration
run_prearchive_invalid_case foreign-path invalid_prearchive_parity_registration

# Adding the lifecycle-bound rows only after archive/finalization is stale:
# their Git introduction must strictly precede the verification evidence.
postarchive_repo="$tmp/postarchive-only-parity-projection"
postarchive_report="$tmp/postarchive-only-parity-report.json"
cp -R "$prearchive_repo" "$postarchive_repo"
python3 - "$postarchive_repo" "$prearchive_staged" <<'PY'
import shutil
import subprocess
import sys
from pathlib import Path

repo = Path(sys.argv[1])
staged = Path(sys.argv[2])
work_item = "WI-901-corrective-after-baseline"
subprocess.run(["git", "reset", "--hard", "HEAD^"], cwd=repo, check=True)
shutil.rmtree(repo / ".ai/work-items/active")
for source in sorted(staged.rglob(f"{work_item}.*.json")):
    relative = source.relative_to(staged)
    destination = repo / relative
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)
subprocess.run(["git", "add", "-A"], cwd=repo, check=True)
subprocess.run(
    ["git", "commit", "-qm", "archive before parity projection"],
    cwd=repo,
    check=True,
)

contract = f".ai/work-items/archive/{work_item}.contract.json"
evidence = f".ai/evidence/{work_item}.verification.json"
finalize = f".ai/decisions/{work_item}.finalize.json"
close = f".ai/decisions/{work_item}.close.json"
statuses = {
    "docs/reference/reference-parity.md": (
        "In progress → Implemented after verified close",
        ";",
    ),
    "docs/reference/reference-parity.zh-CN.md": (
        "进行中 → 验证关闭后已实现",
        "；",
    ),
    "docs/reference/reference-parity.ja.md": (
        "In progress → verified close 後 Implemented",
        "；",
    ),
}
for relative, (status, separator) in statuses.items():
    path = repo / relative
    row = (
        f"| WI-901 — fixture | {status} | "
        f"`{contract}`{separator} `{evidence}`{separator} "
        f"`{finalize}`{separator} `{close}` |"
    )
    path.write_text(path.read_text(encoding="utf-8") + row + "\n", encoding="utf-8")
subprocess.run(["git", "add", "docs/reference"], cwd=repo, check=True)
subprocess.run(
    ["git", "commit", "-qm", "project parity after archive"],
    cwd=repo,
    check=True,
)
PY
set +e
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
  python3 "$gate" --repo "$postarchive_repo" \
    --report "$postarchive_report" >/dev/null
postarchive_code=$?
set -e
[[ "$postarchive_code" -eq 1 ]] || {
  printf 'postarchive parity projection: expected exit 1, got %s\n' \
    "$postarchive_code" >&2
  exit 1
}
python3 - "$postarchive_report" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
findings = [
    finding
    for finding in report["findings"]
    if finding["workItemId"] == "WI-901-corrective-after-baseline"
    and finding["code"] == "stale_prearchive_parity_registration"
]
assert len(findings) == 3, findings
PY

python3 - "$prearchive_repo" "$prearchive_staged" <<'PY'
import json
import shutil
import subprocess
import sys
from pathlib import Path

repo = Path(sys.argv[1])
staged = Path(sys.argv[2])
work_item = "WI-901-corrective-after-baseline"
shutil.rmtree(repo / ".ai/work-items/active")
for source in sorted(staged.rglob(f"{work_item}.*.json")):
    relative = source.relative_to(staged)
    destination = repo / relative
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)
# Archive/evidence are generated before finalization. Keep that lifecycle
# append separate so the finalization receipt binds the exact reviewed
# checkout it observes.
receipt_path = repo / ".ai/decisions" / f"{work_item}.finalize.json"
receipt_path.unlink()
subprocess.run(["git", "add", "-A"], cwd=repo, check=True)
subprocess.run(
    ["git", "commit", "-qm", "archive before finalization"],
    cwd=repo,
    check=True,
)
receipt = json.loads(
    (staged / ".ai/decisions" / f"{work_item}.finalize.json")
    .read_text(encoding="utf-8")
)
reviewed_head = subprocess.check_output(
    ["git", "rev-parse", "HEAD^{commit}"], cwd=repo, text=True
).strip()
receipt["branch"]["headRevision"] = reviewed_head
receipt["pullRequest"]["headRevision"] = reviewed_head
receipt["worktree"]["headRevision"] = reviewed_head
receipt_path.write_text(
    json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8"
)
subprocess.run(["git", "add", str(receipt_path.relative_to(repo))], cwd=repo, check=True)
subprocess.run(
    ["git", "commit", "-qm", "append finalization after archive"],
    cwd=repo,
    check=True,
)
PY
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
  python3 "$gate" --repo "$prearchive_repo" \
    --report "$prearchive_finalize_report" >/dev/null

python3 - "$prearchive_repo" <<'PY'
import json
import subprocess
import sys
from pathlib import Path

repo = Path(sys.argv[1])
work_item = "WI-901-corrective-after-baseline"
evidence = f".ai/evidence/{work_item}.verification.json"
repository_id = json.loads((repo / ".ai/project.json").read_text(encoding="utf-8"))[
    "repositoryId"
]
close = {
    "decisionState": "confirmed",
    "humanDecision": "approved",
    "repositoryId": repository_id,
    "state": "closed",
    "structuredDecision": {
        "actor": "fixture-human",
        "authoritySource": "fixture-policy",
        "decidedAt": "2026-03-02T00:00:00Z",
        "decision": "approved",
        "evidenceRefs": [evidence],
        "policyRefs": ["fixture-policy"],
        "reason": "The exact reviewed resources were cleaned up.",
        "resumeCondition": "None.",
    },
    "workItemId": work_item,
}
path = repo / ".ai/decisions" / f"{work_item}.close.json"
path.write_text(json.dumps(close, indent=2, sort_keys=True) + "\n", encoding="utf-8")
subprocess.run(["git", "add", str(path.relative_to(repo))], cwd=repo, check=True)
subprocess.run(["git", "commit", "-qm", "close without changing parity"], cwd=repo, check=True)
PY
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
  python3 "$gate" --repo "$prearchive_repo" \
    --report "$prearchive_close_report" >/dev/null
python3 - "$prearchive_close_report" "$prearchive_repo" "$prearchive_staged" <<'PY'
import json
import sys
from pathlib import Path

report = json.load(open(sys.argv[1], encoding="utf-8"))
repo = Path(sys.argv[2])
staged = Path(sys.argv[3])
assert report["state"] == "passed", report["findings"]
item = next(
    item
    for item in report["inventory"]
    if item["workItemId"] == "WI-901-corrective-after-baseline"
)
assert item["lifecycleState"] == "closed", item
for path in sorted((staged / "parity").iterdir()):
    assert path.read_bytes() == (repo / "docs/reference" / path.name).read_bytes(), path
PY

# Enriching a row after archive is valid when the same Work Item/status row
# was already registered before its evidence appeared. The history-aware
# check must accept this projection update while the genuinely late-row case
# above remains stale and fail-closed.
enriched_repo="$tmp/postarchive-enriched-parity-projection"
enriched_report="$tmp/postarchive-enriched-parity-report.json"
cp -R "$prearchive_repo" "$enriched_repo"
python3 - "$enriched_repo" <<'PY'
import subprocess
import sys
from pathlib import Path

repo = Path(sys.argv[1])
for relative in (
    "docs/reference/reference-parity.md",
    "docs/reference/reference-parity.zh-CN.md",
    "docs/reference/reference-parity.ja.md",
):
    path = repo / relative
    lines = path.read_text(encoding="utf-8").splitlines()
    updated = []
    for line in lines:
        if line.startswith("| WI-901 "):
            line = line[:-1] + " post-archive evidence projection enrichment |"
        updated.append(line)
    path.write_text("\n".join(updated) + "\n", encoding="utf-8")
subprocess.run(["git", "add", "docs/reference"], cwd=repo, check=True)
subprocess.run(
    ["git", "commit", "-qm", "enrich archived parity projection"],
    cwd=repo,
    check=True,
)
PY
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
  python3 "$gate" --repo "$enriched_repo" --report "$enriched_report" >/dev/null
python3 - "$enriched_report" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
assert report["state"] == "passed", report["findings"]
PY

# A hosted pull-request merge ref combines the feature tree with decisions
# newly present on the default branch.  Every authoritative decision must be
# named by all three parity rows: retaining the pre-merge finalize receipt is
# necessary, but it cannot hide a later close receipt from the merge base.
merge_ref_repo="$tmp/merge-ref-close-parity"
merge_ref_red_report="$tmp/merge-ref-close-parity-red-report.json"
merge_ref_green_report="$tmp/merge-ref-close-parity-green-report.json"
build_fixture "$fixtures/awaiting-merge-close.json" "$merge_ref_repo"
python3 - "$merge_ref_repo" <<'PY'
import json
import subprocess
import sys
from pathlib import Path

repo = Path(sys.argv[1])
work_item = "WI-901-corrective-after-baseline"
evidence = f".ai/evidence/{work_item}.verification.json"
close_path = repo / ".ai/decisions" / f"{work_item}.close.json"

subprocess.run(["git", "checkout", "-q", "codex/fixture"], cwd=repo, check=True)
(repo / "feature-change.txt").write_text("pending parity delivery\n", encoding="utf-8")
subprocess.run(["git", "add", "feature-change.txt"], cwd=repo, check=True)
subprocess.run(["git", "commit", "-qm", "feature parity delivery"], cwd=repo, check=True)

subprocess.run(["git", "checkout", "-q", "main"], cwd=repo, check=True)
repository_id = json.loads((repo / ".ai/project.json").read_text(encoding="utf-8"))[
    "repositoryId"
]
close = {
    "workItemId": work_item,
    "repositoryId": repository_id,
    "state": "closed",
    "decisionState": "confirmed",
    "humanDecision": "approved",
    "structuredDecision": {
        "decision": "approved",
        "actor": "fixture-human",
        "authoritySource": "fixture-policy",
        "reason": "The reviewed pull request merged and exact cleanup was verified.",
        "decidedAt": "2026-03-02T00:00:00Z",
        "resumeCondition": "None.",
        "evidenceRefs": [evidence],
        "policyRefs": ["fixture-policy"],
    },
}
close_path.write_text(json.dumps(close, indent=2, sort_keys=True) + "\n", encoding="utf-8")
subprocess.run(["git", "add", str(close_path.relative_to(repo))], cwd=repo, check=True)
subprocess.run(["git", "commit", "-qm", "default branch close receipt"], cwd=repo, check=True)

subprocess.run(["git", "checkout", "-qb", "merge-ref-red"], cwd=repo, check=True)
subprocess.run(
    ["git", "merge", "--no-ff", "-qm", "hosted red merge ref", "codex/fixture"],
    cwd=repo,
    check=True,
)
PY
set +e
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
  python3 "$gate" --repo "$merge_ref_repo" --report "$merge_ref_red_report" >/dev/null
merge_ref_red_code=$?
set -e
[[ "$merge_ref_red_code" -eq 1 ]] || {
  printf 'merge-ref close parity: expected red exit 1, got %s\n' "$merge_ref_red_code" >&2
  exit 1
}
python3 - "$merge_ref_red_report" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
findings = [
    finding
    for finding in report["findings"]
    if finding["workItemId"] == "WI-901-corrective-after-baseline"
    and finding["code"] == "missing_parity_decision"
]
assert len(findings) == 3, findings
assert {finding["path"] for finding in findings} == {
    "docs/reference/reference-parity.md",
    "docs/reference/reference-parity.zh-CN.md",
    "docs/reference/reference-parity.ja.md",
}, findings
PY
python3 - "$merge_ref_repo" <<'PY'
import subprocess
import sys
from pathlib import Path

repo = Path(sys.argv[1])
work_item = "WI-901-corrective-after-baseline"
finalize = f".ai/decisions/{work_item}.finalize.json"
close = f".ai/decisions/{work_item}.close.json"
subprocess.run(["git", "checkout", "-q", "codex/fixture"], cwd=repo, check=True)
for relative in (
    "docs/reference/reference-parity.md",
    "docs/reference/reference-parity.zh-CN.md",
    "docs/reference/reference-parity.ja.md",
):
    path = repo / relative
    text = path.read_text(encoding="utf-8")
    assert f"`{finalize}`" in text, relative
    separator = "；" if relative != "docs/reference/reference-parity.md" else ";"
    text = text.replace(f"`{finalize}`", f"`{finalize}`{separator} `{close}`")
    path.write_text(text, encoding="utf-8")
subprocess.run(["git", "add", "docs/reference"], cwd=repo, check=True)
subprocess.run(["git", "commit", "-qm", "bind close parity decision"], cwd=repo, check=True)
subprocess.run(["git", "checkout", "-qb", "merge-ref-green", "main"], cwd=repo, check=True)
subprocess.run(
    ["git", "merge", "--no-ff", "-qm", "hosted green merge ref", "codex/fixture"],
    cwd=repo,
    check=True,
)
PY
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
  python3 "$gate" --repo "$merge_ref_repo" --report "$merge_ref_green_report" >/dev/null
python3 - "$merge_ref_green_report" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
assert report["state"] == "passed", report["findings"]
assert not any(
    finding["workItemId"] == "WI-901-corrective-after-baseline"
    and finding["code"] == "missing_parity_decision"
    for finding in report["findings"]
), report["findings"]
PY

# Parity/documentation-owned active Work Items must have all three human
# projections before verification. Ordinary active code Work Items remain
# lightweight; this fixture exercises the dynamic selector and symlink guard.
active_parity_repo="$tmp/active-parity-docs"
active_parity_report="$tmp/active-parity-docs-report.json"
build_fixture "$fixtures/valid.json" "$active_parity_repo"
python3 - "$active_parity_repo" <<'PY'
import json
import os
import sys
from pathlib import Path

repo = Path(sys.argv[1])
work_item = "WI-900-release-v9-9-9"
archive = repo / ".ai/work-items/archive"
active = repo / ".ai/work-items/active"
contract_path = archive / f"{work_item}.contract.json"
contract = json.loads(contract_path.read_text(encoding="utf-8"))
contract["scope"] = ["docs/reference/reference-parity.md"]
contract["acceptanceCriteria"] = ["parity registration is complete"]
active.mkdir(parents=True, exist_ok=True)
(active / f"{work_item}.contract.json").write_text(
    json.dumps(contract, indent=2, sort_keys=True) + "\n", encoding="utf-8"
)
(active / f"{work_item}.summary.json").write_text(
    json.dumps(
        {"workItemId": work_item, "changedPaths": ["docs/reference/reference-parity.md"]},
        indent=2,
        sort_keys=True,
    )
    + "\n",
    encoding="utf-8",
)
for path in archive.glob(f"{work_item}.*.json"):
    path.unlink()
for suffix in ("", ".ja", ".zh-CN"):
    (repo / "docs/work-items" / f"{work_item}{suffix}.md").unlink()
records = (
    f".ai/work-items/archive/{work_item}.contract.json",
    f".ai/evidence/{work_item}.verification.json",
    f".ai/decisions/{work_item}.finalize.json",
    f".ai/decisions/{work_item}.close.json",
)
for relative in (
    "docs/reference/reference-parity.md",
    "docs/reference/reference-parity.zh-CN.md",
    "docs/reference/reference-parity.ja.md",
):
    path = repo / relative
    lines = path.read_text(encoding="utf-8").splitlines()
    status = (
        "进行中 → 验证关闭后已实现"
        if relative.endswith(".zh-CN.md")
        else "In progress → verified close 後 Implemented"
        if relative.endswith(".ja.md")
        else "In progress → Implemented after verified close"
    )
    replacement = (
        f"| WI-900 — fixture | {status} | "
        + "; ".join(f"`{record}`" for record in records)
        + " |"
    )
    path.write_text(
        "\n".join(replacement if line.startswith("| WI-900 ") else line for line in lines)
        + "\n",
        encoding="utf-8",
    )
PY
set +e
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
  python3 "$gate" --repo "$active_parity_repo" --report "$active_parity_report" >/dev/null
active_missing_code=$?
set -e
[[ "$active_missing_code" -eq 1 ]] || {
  printf 'active parity missing docs: expected exit 1, got %s\n' "$active_missing_code" >&2
  exit 1
}
python3 - "$active_parity_report" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
findings = [
    finding
    for finding in report["findings"]
    if finding["workItemId"] == "WI-900-release-v9-9-9"
]
assert [finding["code"] for finding in findings].count("missing_work_item_document") == 3, findings
assert {finding["path"] for finding in findings} == {
    "docs/work-items/WI-900-release-v9-9-9.md",
    "docs/work-items/WI-900-release-v9-9-9.ja.md",
    "docs/work-items/WI-900-release-v9-9-9.zh-CN.md",
}, findings
PY
python3 - "$active_parity_repo" <<'PY'
import sys
from pathlib import Path

repo = Path(sys.argv[1])
work_item = "WI-900-release-v9-9-9"
for suffix in ("", ".ja", ".zh-CN"):
    (repo / "docs/work-items" / f"{work_item}{suffix}.md").write_text(
        "---\n"
        f"workItemId: {work_item}\n"
        "status: in_progress\n"
        "---\n",
        encoding="utf-8",
    )
PY
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
  python3 "$gate" --repo "$active_parity_repo" --report "$active_parity_report" >/dev/null
python3 - "$active_parity_repo/docs/work-items/WI-900-release-v9-9-9.zh-CN.md" <<'PY'
import sys
from pathlib import Path

Path(sys.argv[1]).write_text("malformed projection\n", encoding="utf-8")
PY
set +e
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
  python3 "$gate" --repo "$active_parity_repo" --report "$active_parity_report" >/dev/null
active_malformed_code=$?
set -e
[[ "$active_malformed_code" -eq 1 ]] || {
  printf 'active parity malformed doc: expected exit 1, got %s\n' "$active_malformed_code" >&2
  exit 1
}
python3 - "$active_parity_report" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
assert any(
    finding["workItemId"] == "WI-900-release-v9-9-9"
    and finding["code"] == "invalid_work_item_document"
    and finding["path"].endswith(".zh-CN.md")
    for finding in report["findings"]
), report["findings"]
PY
python3 - "$active_parity_repo/docs/work-items/WI-900-release-v9-9-9.zh-CN.md" <<'PY'
import sys
from pathlib import Path

Path(sys.argv[1]).write_text(
    "---\nworkItemId: WI-900-release-v9-9-9\nstatus: in_progress\n---\n",
    encoding="utf-8",
)
PY
python3 - "$active_parity_repo" <<'PY'
import os
import sys
from pathlib import Path

repo = Path(sys.argv[1])
work_item = "WI-900-release-v9-9-9"
symlink = repo / "docs/work-items" / f"{work_item}.zh-CN.md"
symlink.unlink()
os.symlink(f"{work_item}.md", symlink)
PY
set +e
env -u GITHUB_EVENT_NAME -u GITHUB_REF -u GITHUB_REF_NAME \
  -u GITHUB_SHA -u GITHUB_EVENT_PATH -u GITHUB_BASE_REF \
  python3 "$gate" --repo "$active_parity_repo" --report "$active_parity_report" >/dev/null
active_symlink_code=$?
set -e
[[ "$active_symlink_code" -eq 1 ]] || {
  printf 'active parity symlink doc: expected exit 1, got %s\n' "$active_symlink_code" >&2
  exit 1
}
python3 - "$active_parity_report" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1], encoding="utf-8"))
assert any(
    finding["workItemId"] == "WI-900-release-v9-9-9"
    and finding["code"] == "missing_work_item_document"
    and finding["path"].endswith(".zh-CN.md")
    for finding in report["findings"]
), report["findings"]
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
