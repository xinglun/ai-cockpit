#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/hosted-runtime-verification.XXXXXX")
trap 'rm -rf "$tmp"' EXIT

make_runtime() {
  local mode=$1
  local directory="$tmp/$mode"
  mkdir -p "$directory/repository/.ai/evidence"
  mkdir -p "$directory/repository/.ai/work-items/active"
  printf 'fixture source\n' >"$directory/repository/README.md"
  printf '{"workItemId":"WI-HOSTED-TEST"}\n' >"$directory/repository/.ai/work-items/active/WI-HOSTED-TEST.contract.json"
  : >"$directory/repository/.ai/evidence/.keep"
  git -C "$directory/repository" init -q
  git -C "$directory/repository" config user.name 'Hosted Runtime Test'
  git -C "$directory/repository" config user.email hosted-runtime@example.invalid
  git -C "$directory/repository" add README.md .ai
  git -C "$directory/repository" commit -qm 'hosted Runtime fixture'
  FAKE_MODE="$mode" FAKE_RUNTIME="$directory/runtime" python3 - <<'PY'
import json
import os
from pathlib import Path

path = Path(os.environ["FAKE_RUNTIME"])
mode = os.environ["FAKE_MODE"]
path.write_text(
    "#!/usr/bin/env python3\n"
    "import hashlib, json, os, sys\n"
    "from pathlib import Path\n"
    "args = sys.argv[1:]\n"
    "log = Path(os.environ['FAKE_LOG'])\n"
    "with log.open('a', encoding='utf-8') as stream: stream.write(json.dumps(args) + '\\n')\n"
    "if args[:2] == ['work-item', 'status']:\n"
    "    state = Path(os.environ['FAKE_STATE'])\n"
    "    count = int(state.read_text() if state.exists() else '0') + 1\n"
    "    state.write_text(str(count))\n"
    "    mode = os.environ['FAKE_MODE']\n"
    "    if mode in ('existing-fresh-receipt', 'existing-fresh-mismatched-receipt'):\n"
    "        contract_digest = 'sha256:' + ('4' if mode == 'existing-fresh-mismatched-receipt' else '3') * 64\n"
    "        print(json.dumps({'workItemId': 'WI-HOSTED-TEST', 'verification': 'verified', 'evidenceFreshness': {'state': 'fresh'}, 'repositoryId': 'sha256:' + '1' * 64, 'baseCommit': 'a' * 40, 'sourceDigests': {'contract': contract_digest, 'repositorySnapshot': 'sha256:' + '2' * 64}, 'safeActions': ['run_verification']}))\n"
    "    elif mode == 'stale' and count <= 2: actions = ['run_preflight']\n"
    "    elif mode == 'blocked': actions = []\n"
    "    elif mode == 'blocked-after-preflight' and count >= 3: actions = []\n"
    "    elif mode == 'blocked-after-preflight' and count <= 2: actions = ['run_preflight']\n"
    "    else: actions = ['run_verification']\n"
    "    if mode in ('existing-fresh-receipt', 'existing-fresh-mismatched-receipt'): pass\n"
    "    elif count >= 4:\n"
    "        contract_digest = 'sha256:' + '3' * 64\n"
    "        snapshot_digest = 'sha256:' + '2' * 64\n"
    "        if mode == 'stale-contract-after-verification': contract_digest = 'sha256:' + '4' * 64\n"
    "        if mode == 'stale-snapshot-after-verification': snapshot_digest = 'sha256:' + '4' * 64\n"
    "        print(json.dumps({'workItemId': 'WI-HOSTED-TEST', 'verification': 'verified', 'evidenceFreshness': {'state': 'fresh'}, 'sourceDigests': {'contract': contract_digest, 'repositorySnapshot': snapshot_digest}}))\n"
    "    else:\n"
    "        print(json.dumps({'workItemId': 'WI-HOSTED-TEST', 'safeActions': actions}))\n"
    "elif args and args[0] == 'preflight':\n"
    "    print(json.dumps({'kind': 'preflight', 'state': 'passed'}))\n"
    "elif args and args[0] == 'verify':\n"
    "    assert args[args.index('--workers') + 1] == '2'\n"
    "    repository = Path(args[args.index('--repo') + 1])\n"
    "    runtime_digest = 'sha256:' + hashlib.sha256(Path(sys.argv[0]).read_bytes()).hexdigest()\n"
    "    repository_id = 'sha256:' + '1' * 64\n"
    "    snapshot_digest = 'sha256:' + '2' * 64\n"
    "    contract_digest = 'sha256:' + '3' * 64\n"
    "    receipt = {'workItemId': 'WI-HOSTED-TEST', 'passed': True, 'runtimeDigest': runtime_digest, 'runtimeVersion': '0.2.113', 'repositoryId': repository_id, 'nodesPlanned': 1, 'nodesExecuted': 1, 'nodesReused': 0, 'results': [{'nodeId': 'project-command-0-package-fixture', 'passed': True}], 'planReceipt': {'repositorySnapshotDigest': snapshot_digest}}\n"
    "    evidence = {'workItemId': 'WI-HOSTED-TEST', 'passed': True, 'runtimeDigest': runtime_digest, 'runtimeVersion': '0.2.113', 'contractDigest': contract_digest, 'repositoryId': repository_id, 'repositorySnapshotDigest': snapshot_digest, 'receipt': receipt}\n"
    "    evidence_path = repository / '.ai/evidence/WI-HOSTED-TEST.verification.json'\n"
    "    evidence_path.write_text(json.dumps(evidence), encoding='utf-8')\n"
    "    print(json.dumps({'kind': 'verification', 'passed': True}))\n"
    "else:\n"
    "    raise SystemExit('unexpected fake Runtime command: ' + repr(args))\n",
    encoding="utf-8",
)
path.chmod(0o755)
PY
}

capture_route_receipt() {
  python3 - "$root/tests/ci/quality_route.py" "$1" \
    "$root/tests/ci/repository_gate_manifest.json" "$2" <<'PY'
import importlib.util
import json
import subprocess
import sys
from pathlib import Path

route_path, repository, manifest, output = map(Path, sys.argv[1:])
spec = importlib.util.spec_from_file_location("quality_route", route_path)
assert spec is not None and spec.loader is not None
route = importlib.util.module_from_spec(spec)
spec.loader.exec_module(route)
head = subprocess.check_output(["git", "-C", str(repository), "rev-parse", "HEAD"], text=True).strip()
receipt = route.plan_repository_route(
    repository=repository,
    manifest_path=manifest,
    base=head,
    head=head,
    stage="pull_request",
    risk="normal",
    contract_path=None,
    requested_profile=None,
)
output.write_text(json.dumps(receipt, sort_keys=True) + "\n", encoding="utf-8")
PY
}

run_helper() {
  local mode=$1
  local expect_success=$2
  local directory="$tmp/$mode"
  make_runtime "$mode"
  if [[ "$mode" == existing-fresh-receipt || "$mode" == existing-fresh-mismatched-receipt ]]; then
    python3 - "$directory/runtime" "$directory/repository" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

runtime, repository = map(Path, sys.argv[1:])
work_item_id = "WI-HOSTED-TEST"
runtime_digest = "sha256:" + hashlib.sha256(runtime.read_bytes()).hexdigest()
repository_id = "sha256:" + "1" * 64
contract_digest = "sha256:" + "3" * 64
snapshot_digest = "sha256:" + "2" * 64
node_id = "project-command-0-package-fixture"
reused_node_id = "project-command-1-docs-fixture"
receipt = {
    "workItemId": work_item_id,
    "passed": True,
    "runtimeDigest": runtime_digest,
    "runtimeVersion": "0.2.113",
    "repositoryId": repository_id,
    "nodesPlanned": 2,
    "nodesExecuted": 1,
    "nodesReused": 1,
    "results": [
        {"nodeId": node_id, "passed": True},
        {"nodeId": reused_node_id, "passed": True, "reused": True},
    ],
    "planReceipt": {
        "workItemId": work_item_id,
        "repositoryId": repository_id,
        "baseRevision": "a" * 40,
        "stage": "task",
        "repositorySnapshotDigest": snapshot_digest,
        "executedNodes": [node_id],
        "reusedNodes": [reused_node_id],
        "coverageManifest": {
            "workspaceMembers": ["fixture"],
            "nodeIds": [node_id, reused_node_id],
            "commandDigests": ["sha256:" + "5" * 64, "sha256:" + "6" * 64],
        },
    },
}
formal = {
    "workItemId": work_item_id,
    "passed": True,
    "runtimeDigest": runtime_digest,
    "runtimeVersion": "0.2.113",
    "contractDigest": contract_digest,
    "repositoryId": repository_id,
    "repositorySnapshotDigest": snapshot_digest,
    "receipt": receipt,
}
evidence = repository / ".ai" / "evidence" / f"{work_item_id}.verification.json"
evidence.write_bytes(json.dumps(formal).encode("utf-8"))
PY
  fi
  if [[ "$mode" == stale ]]; then
    capture_route_receipt "$directory/repository" "$directory/route-before.json"
  fi
  local result=0
  FAKE_MODE="$mode" FAKE_LOG="$directory/commands.jsonl" FAKE_STATE="$directory/status-count" \
    "$root/tests/ci/run_hosted_runtime_verification.sh" \
      "$directory/runtime" "$directory/repository" \
      "$directory/repository/.ai/work-items/active/WI-HOSTED-TEST.contract.json" \
      "$directory/artifacts" || result=$?
  if [[ "$expect_success" == true && "$result" != 0 ]]; then
    printf 'hosted Runtime helper failed unexpectedly for mode %s (exit %s)\n' "$mode" "$result" >&2
    return 1
  fi
  if [[ "$expect_success" == false && "$result" == 0 ]]; then
    printf 'hosted Runtime helper accepted blocked mode %s\n' "$mode" >&2
    return 1
  fi
  if [[ "$mode" == stale ]]; then
    capture_route_receipt "$directory/repository" "$directory/route-after.json"
    cmp "$directory/route-before.json" "$directory/route-after.json"
  fi
}

assert_no_commands() {
  local log=$1
  shift
  python3 - "$log" "$@" <<'PY'
import json
import sys
from pathlib import Path

commands = [json.loads(line) for line in Path(sys.argv[1]).read_text().splitlines()]
for command in sys.argv[2:]:
    assert not any(args and args[0] == command for args in commands), (command, commands)
PY
}

run_helper stale true
test -z "$(git -C "$tmp/stale/repository" status --porcelain)" || {
  printf 'hosted verification changed the route-planning checkout\n' >&2
  git -C "$tmp/stale/repository" status --short >&2
  exit 1
}
worktree_count=$(git -C "$tmp/stale/repository" worktree list --porcelain | awk '/^worktree / { count += 1 } END { print count + 0 }')
test "$worktree_count" = 1
jq -e '.kind == "preflight" and .state == "passed"' "$tmp/stale/artifacts/hosted-runtime-preflight.json" >/dev/null
jq -e '.kind == "verification" and .passed == true' "$tmp/stale/artifacts/hosted-runtime-execution.json" >/dev/null
jq -e '.workItemId == "WI-HOSTED-TEST" and .passed == true and .receipt.workItemId == .workItemId and .receipt.passed == true' \
  "$tmp/stale/artifacts/hosted-runtime-verification.json" >/dev/null
jq -e '.state == "removed" and .cleanupExitCode == 0 and .isolatedRepository != null' \
  "$tmp/stale/artifacts/hosted-runtime-worktree-cleanup.json" >/dev/null
python3 - "$tmp/stale/commands.jsonl" <<'PY'
import json
import sys
from pathlib import Path

commands = [json.loads(line) for line in Path(sys.argv[1]).read_text().splitlines()]
assert [command[0] for command in commands] == ["work-item", "work-item", "preflight", "work-item", "verify", "work-item"]
verify = commands[4]
assert "--workers" in verify and verify[verify.index("--workers") + 1] == "2"
assert verify[verify.index("--repo") + 1] != str(Path(sys.argv[1]).parents[0] / "repository")
PY

run_helper fresh true
python3 - "$tmp/fresh/commands.jsonl" <<'PY'
import json
import sys
from pathlib import Path

commands = [json.loads(line) for line in Path(sys.argv[1]).read_text().splitlines()]
assert [command[0] for command in commands] == ["work-item", "work-item", "work-item", "verify", "work-item"]
PY

run_helper existing-fresh-receipt true
cmp "$tmp/existing-fresh-receipt/repository/.ai/evidence/WI-HOSTED-TEST.verification.json" \
  "$tmp/existing-fresh-receipt/artifacts/hosted-runtime-verification.json"
jq -e '.verificationState == "reused" and .preflightState == "not_required_existing_fresh_receipt"' \
  "$tmp/existing-fresh-receipt/artifacts/hosted-runtime-orchestration.json" >/dev/null
assert_no_commands "$tmp/existing-fresh-receipt/commands.jsonl" preflight verify

run_helper existing-fresh-mismatched-receipt false
jq -e '.verificationState == "invalidated" and .failureReason == "existing_fresh_receipt_identity_mismatch"' \
  "$tmp/existing-fresh-mismatched-receipt/artifacts/hosted-runtime-orchestration.json" >/dev/null
assert_no_commands "$tmp/existing-fresh-mismatched-receipt/commands.jsonl" preflight verify

run_helper stale-contract-after-verification false
jq -e '.verificationState == "invalidated" and .failureReason == "post_verification_status_mismatch"' \
  "$tmp/stale-contract-after-verification/artifacts/hosted-runtime-orchestration.json" >/dev/null

run_helper stale-snapshot-after-verification false
jq -e '.verificationState == "invalidated" and .failureReason == "post_verification_status_mismatch"' \
  "$tmp/stale-snapshot-after-verification/artifacts/hosted-runtime-orchestration.json" >/dev/null

run_helper blocked false
assert_no_commands "$tmp/blocked/commands.jsonl" preflight verify

run_helper blocked-after-preflight false
assert_no_commands "$tmp/blocked-after-preflight/commands.jsonl" verify

printf 'hosted Runtime verification orchestration regression passed\n'
