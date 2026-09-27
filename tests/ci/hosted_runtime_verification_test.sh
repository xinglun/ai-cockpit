#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/hosted-runtime-verification.XXXXXX")
trap 'rm -rf "$tmp"' EXIT

cat >"$tmp/contract.json" <<'JSON'
{"workItemId":"WI-HOSTED-TEST"}
JSON

make_runtime() {
  local mode=$1
  local directory="$tmp/$mode"
  mkdir -p "$directory/repository/.ai/evidence"
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
    "    if mode == 'stale' and count == 1: actions = ['run_preflight']\n"
    "    elif mode == 'blocked': actions = []\n"
    "    elif mode == 'blocked-after-preflight' and count > 1: actions = []\n"
    "    else: actions = ['run_verification']\n"
    "    print(json.dumps({'workItemId': 'WI-HOSTED-TEST', 'safeActions': actions}))\n"
    "elif args and args[0] == 'preflight':\n"
    "    print(json.dumps({'kind': 'preflight', 'state': 'passed'}))\n"
    "elif args and args[0] == 'verify':\n"
    "    assert args[args.index('--workers') + 1] == '1'\n"
    "    repository = Path(args[args.index('--repo') + 1])\n"
    "    runtime_digest = 'sha256:' + hashlib.sha256(Path(sys.argv[0]).read_bytes()).hexdigest()\n"
    "    repository_id = 'sha256:' + '1' * 64\n"
    "    snapshot_digest = 'sha256:' + '2' * 64\n"
    "    receipt = {'workItemId': 'WI-HOSTED-TEST', 'passed': True, 'runtimeDigest': runtime_digest, 'runtimeVersion': '0.2.113', 'repositoryId': repository_id, 'nodesPlanned': 1, 'results': [{'nodeId': 'project-command-0-package-fixture', 'passed': True}], 'planReceipt': {'repositorySnapshotDigest': snapshot_digest}}\n"
    "    evidence = {'workItemId': 'WI-HOSTED-TEST', 'passed': True, 'runtimeDigest': runtime_digest, 'runtimeVersion': '0.2.113', 'repositoryId': repository_id, 'repositorySnapshotDigest': snapshot_digest, 'receipt': receipt}\n"
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

run_helper() {
  local mode=$1
  local expect_success=$2
  local directory="$tmp/$mode"
  make_runtime "$mode"
  local result=0
  FAKE_MODE="$mode" FAKE_LOG="$directory/commands.jsonl" FAKE_STATE="$directory/status-count" \
    "$root/tests/ci/run_hosted_runtime_verification.sh" \
      "$directory/runtime" "$directory/repository" "$tmp/contract.json" "$directory/artifacts" || result=$?
  if [[ "$expect_success" == true && "$result" != 0 ]]; then
    printf 'hosted Runtime helper failed unexpectedly for mode %s (exit %s)\n' "$mode" "$result" >&2
    return 1
  fi
  if [[ "$expect_success" == false && "$result" == 0 ]]; then
    printf 'hosted Runtime helper accepted blocked mode %s\n' "$mode" >&2
    return 1
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
jq -e '.kind == "preflight" and .state == "passed"' "$tmp/stale/artifacts/hosted-runtime-preflight.json" >/dev/null
jq -e '.kind == "verification" and .passed == true' "$tmp/stale/artifacts/hosted-runtime-execution.json" >/dev/null
jq -e '.workItemId == "WI-HOSTED-TEST" and .passed == true and .receipt.workItemId == .workItemId and .receipt.passed == true' \
  "$tmp/stale/artifacts/hosted-runtime-verification.json" >/dev/null
cmp "$tmp/stale/repository/.ai/evidence/WI-HOSTED-TEST.verification.json" \
  "$tmp/stale/artifacts/hosted-runtime-verification.json"
python3 - "$tmp/stale/commands.jsonl" <<'PY'
import json
import sys
from pathlib import Path

commands = [json.loads(line) for line in Path(sys.argv[1]).read_text().splitlines()]
assert [command[0] for command in commands] == ["work-item", "preflight", "work-item", "verify"]
verify = commands[-1]
assert "--workers" in verify and verify[verify.index("--workers") + 1] == "1"
PY

run_helper fresh true
python3 - "$tmp/fresh/commands.jsonl" <<'PY'
import json
import sys
from pathlib import Path

commands = [json.loads(line) for line in Path(sys.argv[1]).read_text().splitlines()]
assert [command[0] for command in commands] == ["work-item", "work-item", "verify"]
PY

run_helper blocked false
assert_no_commands "$tmp/blocked/commands.jsonl" preflight verify

run_helper blocked-after-preflight false
assert_no_commands "$tmp/blocked-after-preflight/commands.jsonl" verify

printf 'hosted Runtime verification orchestration regression passed\n'
