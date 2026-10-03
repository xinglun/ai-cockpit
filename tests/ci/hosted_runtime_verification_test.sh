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
  printf '{"workItemId":"WI-HOSTED-TEST","verification":["cargo test --locked --workspace"]}\n' >"$directory/repository/.ai/work-items/active/WI-HOSTED-TEST.contract.json"
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
    "if args[:2] == ['work-item', 'validate']:\n"
    "    mode = os.environ['FAKE_MODE']\n"
    "    if mode == 'invalid-validation': print(json.dumps({'state': 'blocked'}))\n"
    "    else: print(json.dumps({'state': 'blocked', 'unknowns': [], 'findings': []}))\n"
    "elif args[:2] == ['work-item', 'status']:\n"
    "    state = Path(os.environ['FAKE_STATE'])\n"
    "    count = int(state.read_text() if state.exists() else '0') + 1\n"
    "    state.write_text(str(count))\n"
    "    mode = os.environ['FAKE_MODE']\n"
    "    if mode in ('existing-fresh-receipt', 'existing-fresh-mismatched-receipt'):\n"
    "        repository = Path(args[args.index('--repo') + 1])\n"
    "        contract = json.loads((repository / '.ai/work-items/active/WI-HOSTED-TEST.contract.json').read_text())\n"
    "        contract_digest = 'sha256:' + hashlib.sha256(json.dumps(contract, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()\n"
    "        if mode == 'existing-fresh-mismatched-receipt': contract_digest = 'sha256:' + '4' * 64\n"
    "        print(json.dumps({'workItemId': 'WI-HOSTED-TEST', 'verification': 'verified', 'evidenceFreshness': {'state': 'fresh'}, 'repositoryId': 'sha256:' + '1' * 64, 'baseCommit': 'a' * 40, 'sourceDigests': {'contract': contract_digest, 'repositorySnapshot': 'sha256:' + '2' * 64}, 'safeActions': ['run_verification']}))\n"
    "    elif mode in ('stale', 'stale-deferred') and count <= 2: actions = ['run_preflight']\n"
    "    elif mode == 'blocked': actions = []\n"
    "    elif mode == 'blocked-after-preflight' and count >= 3: actions = []\n"
    "    elif mode == 'blocked-after-preflight' and count <= 2: actions = ['run_preflight']\n"
    "    else: actions = ['run_verification']\n"
    "    if mode in ('existing-fresh-receipt', 'existing-fresh-mismatched-receipt'): pass\n"
    "    elif count >= 4:\n"
    "        repository = Path(args[args.index('--repo') + 1])\n"
    "        contract = json.loads((repository / '.ai/work-items/active/WI-HOSTED-TEST.contract.json').read_text())\n"
    "        contract_digest = 'sha256:' + hashlib.sha256(json.dumps(contract, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()\n"
    "        snapshot_digest = 'sha256:' + '2' * 64\n"
    "        if mode == 'stale-contract-after-verification': contract_digest = 'sha256:' + '4' * 64\n"
    "        if mode == 'stale-snapshot-after-verification': snapshot_digest = 'sha256:' + '4' * 64\n"
    "        print(json.dumps({'workItemId': 'WI-HOSTED-TEST', 'verification': 'verified', 'evidenceFreshness': {'state': 'fresh'}, 'sourceDigests': {'contract': contract_digest, 'repositorySnapshot': snapshot_digest}}))\n"
    "    else:\n"
    "        print(json.dumps({'workItemId': 'WI-HOSTED-TEST', 'safeActions': actions}))\n"
    "elif args and args[0] == 'preflight':\n"
    "    print(json.dumps({'kind': 'preflight', 'state': 'passed'}))\n"
    "elif args and args[0] == 'verify':\n"
    "    assert args[args.index('--workers') + 1] == '1'\n"
    "    repository = Path(args[args.index('--repo') + 1])\n"
    "    runtime_digest = 'sha256:' + hashlib.sha256(Path(sys.argv[0]).read_bytes()).hexdigest()\n"
    "    repository_id = 'sha256:' + '1' * 64\n"
    "    snapshot_digest = 'sha256:' + '2' * 64\n"
    "    contract = json.loads((repository / '.ai/work-items/active/WI-HOSTED-TEST.contract.json').read_text())\n"
    "    contract_digest = 'sha256:' + hashlib.sha256(json.dumps(contract, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()\n"
    "    node_id = 'project-command-0-package-fixture'\n"
    "    command_digest = 'sha256:' + hashlib.sha256(b'cargo test --locked --workspace').hexdigest()\n"
    "    receipt = {'workItemId': 'WI-HOSTED-TEST', 'passed': True, 'runtimeDigest': runtime_digest, 'runtimeVersion': '0.2.113', 'repositoryId': repository_id, 'nodesPlanned': 1, 'nodesExecuted': 1, 'nodesReused': 0, 'results': [{'nodeId': node_id, 'passed': True, 'reused': False, 'action': 'execute', 'satisfiedBy': 'execution'}], 'executionRecords': [{'nodeId': node_id, 'commandDigest': command_digest, 'spawned': True, 'passed': True, 'exitCode': 0}], 'planReceipt': {'workItemId': 'WI-HOSTED-TEST', 'repositoryId': repository_id, 'baseRevision': 'a' * 40, 'stage': 'task', 'repositorySnapshotDigest': snapshot_digest, 'executedNodes': [node_id], 'reusedNodes': [], 'coverageManifest': {'workspaceMembers': ['fixture'], 'nodeIds': [node_id], 'commandDigests': [command_digest], 'sourceProgram': 'cargo', 'sourceArgs': ['test', '--locked', '--workspace']}}}\n"
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
  local defer_worktree=${3:-false}
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
contract = json.loads((repository / ".ai/work-items/active/WI-HOSTED-TEST.contract.json").read_text())
contract_digest = "sha256:" + hashlib.sha256(
    json.dumps(contract, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
).hexdigest()
snapshot_digest = "sha256:" + "2" * 64
node_id = "project-command-0-package-fixture"
reused_node_id = "project-command-1-docs-fixture"
command_digest = "sha256:" + hashlib.sha256(b"cargo test --locked --workspace").hexdigest()
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
        {"nodeId": node_id, "passed": True, "reused": False, "action": "execute", "satisfiedBy": "execution"},
        {"nodeId": reused_node_id, "passed": True, "reused": True, "action": "reuse", "satisfiedBy": "reused_receipt"},
    ],
    "executionRecords": [{"nodeId": node_id, "commandDigest": command_digest, "spawned": True, "passed": True, "exitCode": 0}],
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
            "nodeIds": [node_id],
            "commandDigests": [command_digest],
            "sourceProgram": "cargo",
            "sourceArgs": ["test", "--locked", "--workspace"],
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
  mkdir -p "$directory/runner-temp"
  local result=0
  if [[ "$defer_worktree" == true ]]; then
    RUNNER_TEMP="$directory/runner-temp" AI_COCKPIT_DEFER_WORKTREE_CLEANUP=true \
      FAKE_MODE="$mode" FAKE_LOG="$directory/commands.jsonl" FAKE_STATE="$directory/status-count" \
      "$root/tests/ci/run_hosted_runtime_verification.sh" \
        "$directory/runtime" "$directory/repository" \
        "$directory/repository/.ai/work-items/active/WI-HOSTED-TEST.contract.json" \
        "$directory/artifacts" || result=$?
  else
    FAKE_MODE="$mode" FAKE_LOG="$directory/commands.jsonl" FAKE_STATE="$directory/status-count" \
      "$root/tests/ci/run_hosted_runtime_verification.sh" \
        "$directory/runtime" "$directory/repository" \
        "$directory/repository/.ai/work-items/active/WI-HOSTED-TEST.contract.json" \
        "$directory/artifacts" || result=$?
  fi
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
assert [command[0] for command in commands] == ["work-item", "work-item", "work-item", "preflight", "work-item", "work-item", "verify", "work-item"]
verify = commands[6]
assert "--workers" in verify and verify[verify.index("--workers") + 1] == "1"
assert verify[verify.index("--repo") + 1] != str(Path(sys.argv[1]).parents[0] / "repository")
PY
jq -e '.state == "blocked" and (.unknowns | type == "array") and (.findings | type == "array")' \
  "$tmp/stale/artifacts/hosted-runtime-validation-after.json" >/dev/null

# The producer must keep a newly generated isolated receipt available through
# its downstream coverage consumer, rather than deleting its source worktree
# as soon as verification exits.
run_helper stale-deferred true true
jq -e '.state == "deferred_for_consumer" and .cleanupExitCode == 0 and .isolatedRepository != null' \
  "$tmp/stale-deferred/artifacts/hosted-runtime-worktree-cleanup.json" >/dev/null
deferred_repository=$(jq -er '.isolatedRepository' \
  "$tmp/stale-deferred/artifacts/hosted-runtime-worktree-cleanup.json")
test -d "$deferred_repository"
git -C "$tmp/stale-deferred/repository" worktree list --porcelain | \
  grep -F -q "worktree $deferred_repository"
jq -e --arg repository "$deferred_repository" '.executionRepository == $repository and .verificationState == "passed"' \
  "$tmp/stale-deferred/artifacts/hosted-runtime-orchestration.json" >/dev/null
resolved_repository=$(RUNNER_TEMP="$tmp/stale-deferred/runner-temp" \
  "$root/tests/ci/cleanup_hosted_runtime_verification_worktree.sh" --resolve \
    "$tmp/stale-deferred/repository" \
    "$tmp/stale-deferred/artifacts/hosted-runtime-worktree-cleanup.json")
test "$resolved_repository" = "$deferred_repository"
test -d "$deferred_repository"
python3 - \
  "$tmp/stale-deferred/artifacts/hosted-runtime-verification.json" \
  "$tmp/stale-deferred/artifacts/hosted-runtime-orchestration.json" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

receipt_path, orchestration_path = map(Path, sys.argv[1:])
orchestration = json.loads(orchestration_path.read_text(encoding="utf-8"))
assert orchestration["formalReceiptDigest"] == "sha256:" + hashlib.sha256(receipt_path.read_bytes()).hexdigest()
PY
python3 - "$tmp/stale-deferred/metadata.json" <<'PY'
import json
import sys
from pathlib import Path

Path(sys.argv[1]).write_text(json.dumps({"packages": [{"name": "fixture", "source": None}]}), encoding="utf-8")
PY
python3 - "$tmp/stale-deferred/fake-cargo" <<'PY'
import os
import sys
from pathlib import Path

path = Path(sys.argv[1])
path.write_text(
    "#!/usr/bin/env python3\n"
    "import os, sys\n"
    "with open(os.environ['PACKAGE_LOG'], 'a', encoding='utf-8') as stream: stream.write('fixture\\n')\n",
    encoding="utf-8",
)
path.chmod(0o755)
PY
PACKAGE_LOG="$tmp/stale-deferred/packages.log" \
FAKE_MODE=stale-deferred \
FAKE_LOG="$tmp/stale-deferred/commands.jsonl" \
FAKE_STATE="$tmp/stale-deferred/status-count" \
AI_COCKPIT_VERIFICATION_RECEIPT="$tmp/stale-deferred/artifacts/hosted-runtime-verification.json" \
AI_COCKPIT_VERIFICATION_ORCHESTRATION="$tmp/stale-deferred/artifacts/hosted-runtime-orchestration.json" \
AI_COCKPIT_RUNTIME_BIN="$tmp/stale-deferred/runtime" \
AI_COCKPIT_VERIFICATION_REPOSITORY="$deferred_repository" \
RUNNER_TEMP="$tmp/stale-deferred/runner-temp" \
  "$root/tests/ci/run_workspace_package_tests.sh" \
    --metadata "$tmp/stale-deferred/metadata.json" \
    --cargo "$tmp/stale-deferred/fake-cargo" \
    --report "$tmp/stale-deferred/coverage.json"
test ! -e "$tmp/stale-deferred/packages.log"
jq -e '.state == "passed" and .verificationReceiptState == "passed" and .executedByHostedRuntime == ["fixture"] and .executedByCoverageRunner == []' \
  "$tmp/stale-deferred/coverage.json" >/dev/null

# A malformed formal receipt must fail before the package-test Cargo process.
# Mutate only this temporary fake Runtime evidence, then restore its passing
# bytes so the valid fixture and deferred cleanup remain independently usable.
assert_rejected_coverage_receipt() {
  local case_name=$1
  local receipt="$tmp/stale-deferred/artifacts/hosted-runtime-verification.json"
  local orchestration="$tmp/stale-deferred/artifacts/hosted-runtime-orchestration.json"
  local evidence="$deferred_repository/.ai/evidence/WI-HOSTED-TEST.verification.json"
  local contract="$deferred_repository/.ai/work-items/active/WI-HOSTED-TEST.contract.json"
  local case_dir="$tmp/stale-deferred/$case_name"
  mkdir -p "$case_dir"
  cp "$contract" "$case_dir/original-contract.json"
  python3 - "$receipt" "$orchestration" "$evidence" "$contract" "$case_dir" "$case_name" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

receipt_path, orchestration_path, evidence_path, contract_path, case_dir = map(Path, sys.argv[1:6])
case_name = sys.argv[6]
formal = json.loads(receipt_path.read_text(encoding="utf-8"))
if case_name == "missing-source-args":
    del formal["receipt"]["planReceipt"]["coverageManifest"]["sourceArgs"]
elif case_name == "missing-contract-declaration":
    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    contract["verification"] = []
    contract_path.write_text(json.dumps(contract), encoding="utf-8")
    formal["contractDigest"] = "sha256:" + hashlib.sha256(
        json.dumps(contract, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
    ).hexdigest()
elif case_name == "missing-execution-record":
    formal["receipt"]["executionRecords"] = []
elif case_name == "wrong-execution-digest":
    formal["receipt"]["executionRecords"][0]["commandDigest"] = "sha256:" + "f" * 64
else:
    raise SystemExit(f"unsupported fixture case: {case_name}")
altered = json.dumps(formal).encode("utf-8")
(case_dir / "verification.json").write_bytes(altered)
evidence_path.write_bytes(altered)
orchestration = json.loads(orchestration_path.read_text(encoding="utf-8"))
orchestration["formalReceiptDigest"] = "sha256:" + hashlib.sha256(altered).hexdigest()
(case_dir / "orchestration.json").write_text(json.dumps(orchestration), encoding="utf-8")
PY
  local result=0
  PACKAGE_LOG="$case_dir/packages.log" \
  FAKE_MODE=stale-deferred \
  FAKE_LOG="$tmp/stale-deferred/commands.jsonl" \
  FAKE_STATE="$tmp/stale-deferred/status-count" \
  AI_COCKPIT_VERIFICATION_RECEIPT="$case_dir/verification.json" \
  AI_COCKPIT_VERIFICATION_ORCHESTRATION="$case_dir/orchestration.json" \
  AI_COCKPIT_RUNTIME_BIN="$tmp/stale-deferred/runtime" \
  AI_COCKPIT_VERIFICATION_REPOSITORY="$deferred_repository" \
    "$root/tests/ci/run_workspace_package_tests.sh" \
      --metadata "$tmp/stale-deferred/metadata.json" \
      --cargo "$tmp/stale-deferred/fake-cargo" \
      --report "$case_dir/coverage.json" >"$case_dir/coverage.log" 2>&1 || result=$?
  cp "$receipt" "$evidence"
  cp "$case_dir/original-contract.json" "$contract"
  if [[ "$result" == 0 ]]; then
    printf 'coverage accepted malformed hosted receipt: %s\n' "$case_name" >&2
    return 1
  fi
  test ! -e "$case_dir/packages.log"
  jq -e '.state == "failed" and .failurePhase == "hosted_verification_receipt" and .executed == []' \
    "$case_dir/coverage.json" >/dev/null
}
assert_rejected_coverage_receipt missing-source-args
assert_rejected_coverage_receipt missing-contract-declaration
assert_rejected_coverage_receipt missing-execution-record
assert_rejected_coverage_receipt wrong-execution-digest

cp "$tmp/stale-deferred/artifacts/hosted-runtime-worktree-cleanup.json" \
  "$tmp/stale-deferred/artifacts/tampered-worktree-cleanup.json"
mkdir -p "$tmp/stale-deferred/runner-temp/unowned/source"
python3 - "$tmp/stale-deferred/artifacts/tampered-worktree-cleanup.json" "$tmp/stale-deferred/runner-temp" <<'PY'
import json
import sys
from pathlib import Path

record_path = Path(sys.argv[1])
record = json.loads(record_path.read_text(encoding="utf-8"))
record["isolatedRepository"] = str(Path(sys.argv[2]) / "unowned" / "source")
record_path.write_text(json.dumps(record), encoding="utf-8")
PY
if RUNNER_TEMP="$tmp/stale-deferred/runner-temp" \
  "$root/tests/ci/cleanup_hosted_runtime_verification_worktree.sh" \
    "$tmp/stale-deferred/repository" \
    "$tmp/stale-deferred/artifacts/tampered-worktree-cleanup.json" \
    >"$tmp/stale-deferred/unsafe-cleanup.log" 2>&1; then
  printf 'cleanup accepted a path outside its dedicated temporary parent\n' >&2
  exit 1
fi
if ! grep -F -q 'outside its dedicated hosted Runtime temporary parent' \
  "$tmp/stale-deferred/unsafe-cleanup.log"; then
  cat "$tmp/stale-deferred/unsafe-cleanup.log" >&2
  exit 1
fi
jq -e '.state == "deferred_for_consumer"' \
  "$tmp/stale-deferred/artifacts/tampered-worktree-cleanup.json" >/dev/null
RUNNER_TEMP="$tmp/stale-deferred/runner-temp" \
  "$root/tests/ci/cleanup_hosted_runtime_verification_worktree.sh" \
    "$tmp/stale-deferred/repository" \
    "$tmp/stale-deferred/artifacts/hosted-runtime-worktree-cleanup.json"
jq -e '.state == "removed" and .cleanupExitCode == 0' \
  "$tmp/stale-deferred/artifacts/hosted-runtime-worktree-cleanup.json" >/dev/null
test ! -e "$deferred_repository"
if git -C "$tmp/stale-deferred/repository" worktree list --porcelain | \
  grep -F -q "worktree $deferred_repository"; then
  printf 'consumer cleanup left the linked worktree registered\n' >&2
  exit 1
fi

run_helper fresh true
python3 - "$tmp/fresh/commands.jsonl" <<'PY'
import json
import sys
from pathlib import Path

commands = [json.loads(line) for line in Path(sys.argv[1]).read_text().splitlines()]
assert [command[0] for command in commands] == ["work-item", "work-item", "work-item", "work-item", "work-item", "verify", "work-item"]
PY

run_helper existing-fresh-receipt true
cmp "$tmp/existing-fresh-receipt/repository/.ai/evidence/WI-HOSTED-TEST.verification.json" \
  "$tmp/existing-fresh-receipt/artifacts/hosted-runtime-verification.json"
jq -e '.verificationState == "reused" and .preflightState == "not_required_existing_fresh_receipt"' \
  "$tmp/existing-fresh-receipt/artifacts/hosted-runtime-orchestration.json" >/dev/null
assert_no_commands "$tmp/existing-fresh-receipt/commands.jsonl" preflight verify
jq -e '.state == "blocked" and (.unknowns | type == "array") and (.findings | type == "array")' \
  "$tmp/existing-fresh-receipt/artifacts/hosted-runtime-validation-before.json" >/dev/null

run_helper existing-fresh-mismatched-receipt false
jq -e '.verificationState == "invalidated" and .failureReason == "existing_fresh_receipt_identity_mismatch"' \
  "$tmp/existing-fresh-mismatched-receipt/artifacts/hosted-runtime-orchestration.json" >/dev/null
assert_no_commands "$tmp/existing-fresh-mismatched-receipt/commands.jsonl" preflight verify

run_helper stale-contract-after-verification false true
jq -e '.verificationState == "invalidated" and .failureReason == "post_verification_status_mismatch"' \
  "$tmp/stale-contract-after-verification/artifacts/hosted-runtime-orchestration.json" >/dev/null
jq -e '.state == "removed" and .cleanupExitCode == 0' \
  "$tmp/stale-contract-after-verification/artifacts/hosted-runtime-worktree-cleanup.json" >/dev/null
test "$(git -C "$tmp/stale-contract-after-verification/repository" worktree list --porcelain | awk '/^worktree / { count += 1 } END { print count + 0 }')" = 1

run_helper stale-snapshot-after-verification false
jq -e '.verificationState == "invalidated" and .failureReason == "post_verification_status_mismatch"' \
  "$tmp/stale-snapshot-after-verification/artifacts/hosted-runtime-orchestration.json" >/dev/null

run_helper blocked false
assert_no_commands "$tmp/blocked/commands.jsonl" preflight verify

run_helper blocked-after-preflight false
assert_no_commands "$tmp/blocked-after-preflight/commands.jsonl" verify

run_helper invalid-validation false
jq -e '.verificationState == "invalidated" and .failureReason == "validation_report_invalid"' \
  "$tmp/invalid-validation/artifacts/hosted-runtime-orchestration.json" >/dev/null
assert_no_commands "$tmp/invalid-validation/commands.jsonl" verify

printf 'hosted Runtime verification orchestration regression passed\n'
