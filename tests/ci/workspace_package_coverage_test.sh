#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/workspace-package-coverage.XXXXXX")
trap 'rm -rf "$tmp"' EXIT

# This regression runs fake receipt and cargo fixtures. Do not inherit the
# hosted verification inputs exported by the enclosing CI gate invocation.
unset AI_COCKPIT_VERIFICATION_RECEIPT \
  AI_COCKPIT_VERIFICATION_ORCHESTRATION \
  AI_COCKPIT_RUNTIME_BIN \
  AI_COCKPIT_VERIFICATION_REPOSITORY

# The enclosing hosted job may point Cargo at the source checkout's runner.
# These fixture subprocesses must use the runner in their own checkout.
host_target=$("${RUSTC:-rustc}" -vV | sed -n 's/^host: //p')
test -n "$host_target"
runner_variable="CARGO_TARGET_${host_target//-/_}_RUNNER"
runner_variable=$(tr '[:lower:]' '[:upper:]' <<<"$runner_variable")
export "${runner_variable}=$root/tests/ci/run_process_observer_test_runner.sh"

cat >"$tmp/metadata.json" <<'JSON'
{"packages":[
  {"name":"package-b","source":null,"version":"1.0.0","targets":[{"name":"package_b","kind":["lib"],"test":true}]},
  {"name":"external","source":"registry+example","version":"1.0.0"},
  {"name":"package-a","source":null,"version":"1.0.0","targets":[{"name":"package_a","kind":["lib"],"test":true}]}
]}
JSON
cat >"$tmp/fake-cargo" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$3" >>"$PACKAGE_LOG"
SH
chmod +x "$tmp/fake-cargo"

PACKAGE_LOG="$tmp/packages.log" WORKSPACE_TEST_WORKERS=1 WORKSPACE_TEST_THREADS=1 "$root/tests/ci/run_workspace_package_tests.sh" \
  --metadata "$tmp/metadata.json" --cargo "$tmp/fake-cargo" --report "$tmp/report.json"
diff -u <(printf '%s\n' package-a package-b) "$tmp/packages.log"
jq -e '.state == "passed" and .planned == ["package-a", "package-b"] and .executed == .planned' "$tmp/report.json" >/dev/null

# Parallel completion must not make the machine-readable report depend on
# which package happened to finish first.
cat >"$tmp/out-of-order-cargo" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$3" == package-a ]]; then sleep 0.05; fi
printf '%s\n' "$3" >>"$PACKAGE_LOG"
SH
chmod +x "$tmp/out-of-order-cargo"
PACKAGE_LOG="$tmp/out-of-order-packages.log" WORKSPACE_TEST_WORKERS=2 WORKSPACE_TEST_THREADS=2 \
  "$root/tests/ci/run_workspace_package_tests.sh" --metadata "$tmp/metadata.json" \
  --cargo "$tmp/out-of-order-cargo" --report "$tmp/out-of-order-report.json"
jq -e '.state == "passed" and .planned == ["package-a", "package-b"] and .executed == .planned' \
  "$tmp/out-of-order-report.json" >/dev/null

# Preserve the first useful diagnosis when one package fails.  The text is
# intentionally similar to an unrelated inventory mention so the repository
# gate classifier cannot mistake an arbitrary package failure for a conformance
# ledger failure.
cat >"$tmp/failing-diagnostic-cargo" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$3" == package-a ]]; then
  printf 'test oversized_reference_inventory_uses_its_strict_conformance_gate ... FAILED\n' >&2
  printf 'assertion failed: package fixture failure\n' >&2
  exit 17
fi
SH
chmod +x "$tmp/failing-diagnostic-cargo"
if WORKSPACE_TEST_WORKERS=1 WORKSPACE_TEST_THREADS=1 "$root/tests/ci/run_workspace_package_tests.sh" \
  --metadata "$tmp/metadata.json" --cargo "$tmp/failing-diagnostic-cargo" --report "$tmp/diagnostic-report.json" \
  >/dev/null 2>&1; then
  printf 'workspace coverage accepted a diagnostic package failure\n' >&2
  exit 1
fi
jq -e '.state == "failed" and .failedPackage == "package-a" and .failedExitCode == 17 and (.failureDiagnosticTail | contains("oversized_reference_inventory"))' \
  "$tmp/diagnostic-report.json" >/dev/null

# A failed package must stop the run and produce a fail-closed receipt that
# exposes the omitted remainder.
cat >"$tmp/failing-cargo" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
[[ "$3" != package-a ]]
SH
chmod +x "$tmp/failing-cargo"
if WORKSPACE_TEST_WORKERS=1 WORKSPACE_TEST_THREADS=1 "$root/tests/ci/run_workspace_package_tests.sh" --metadata "$tmp/metadata.json" \
  --cargo "$tmp/failing-cargo" --report "$tmp/failing-report.json" >/dev/null 2>&1; then
  printf 'workspace coverage accepted an omitted package\n' >&2
  exit 1
fi
jq -e '.state == "failed" and .executed == [] and .omitted == ["package-a", "package-b"]' "$tmp/failing-report.json" >/dev/null

# Metadata discovery itself is part of the fail-closed coverage boundary. A
# cargo metadata launch/failure must still leave a machine-readable receipt.
cat >"$tmp/failing-metadata-cargo" <<'SH'
#!/usr/bin/env bash
exit 42
SH
chmod +x "$tmp/failing-metadata-cargo"
if WORKSPACE_TEST_WORKERS=1 WORKSPACE_TEST_THREADS=1 "$root/tests/ci/run_workspace_package_tests.sh" --cargo "$tmp/failing-metadata-cargo" \
  --report "$tmp/failing-metadata-report.json" >/dev/null 2>&1; then
  printf 'workspace coverage accepted failed cargo metadata\n' >&2
  exit 1
fi
jq -e '.state == "failed" and .failurePhase == "metadata" and .planned == [] and .executed == []' \
  "$tmp/failing-metadata-report.json" >/dev/null

# A hosted Runtime verification receipt can supply the already-executed
# workspace package results, but only when it is bound to the exact executable
# and matching formal evidence. This prevents the package tests from running a
# second time after the Contract gate consumes that receipt.
mkdir -p "$tmp/hosted-repository/.ai/evidence" "$tmp/hosted-repository/.ai/work-items/active"
python3 - "$tmp/metadata.json" "$tmp/runtime-bin" "$tmp/hosted-verification.json" "$tmp/hosted-repository" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

metadata_path, runtime_path, receipt_path, repository = map(Path, sys.argv[1:])
runtime_path.write_text(
    "#!/usr/bin/env python3\n"
    "import json, os, sys\n"
    "from pathlib import Path\n"
    "args = sys.argv[1:]\n"
    "with Path(os.environ['FAKE_RUNTIME_LOG']).open('a', encoding='utf-8') as log: log.write(json.dumps(args) + '\\n')\n"
    "expected = ['work-item', 'status', '--repo', str(Path(os.environ['AI_COCKPIT_VERIFICATION_REPOSITORY'])), '--id', 'WI-HOSTED-RECEIPT', '--json']\n"
    "if args != expected: raise SystemExit('unexpected Runtime status query: ' + repr(args) + '; expected: ' + repr(expected))\n"
    "print(json.dumps({'workItemId': 'WI-HOSTED-RECEIPT', 'verification': 'verified', 'evidenceFreshness': {'state': os.environ.get('FAKE_FRESHNESS_STATE', 'fresh')}, 'sourceDigests': {'contract': os.environ['FAKE_CURRENT_CONTRACT_DIGEST'], 'repositorySnapshot': os.environ['FAKE_CURRENT_SNAPSHOT_DIGEST']}}))\n",
    encoding="utf-8",
)
runtime_path.chmod(0o755)
metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
packages = sorted({p["name"] for p in metadata["packages"] if p.get("source") is None})
runtime_digest = "sha256:" + hashlib.sha256(runtime_path.read_bytes()).hexdigest()
repository_id = "sha256:" + "1" * 64
snapshot_digest = "sha256:" + "2" * 64
work_item_id = "WI-HOSTED-RECEIPT"
command_digest = "sha256:" + "4" * 64
test_nodes = [f"project-command-2-package-{package}" for package in packages]
lint_nodes = [f"project-command-1-package-{package}" for package in packages]
(repository / ".ai/work-items/active" / f"{work_item_id}.contract.json").write_text(
    json.dumps({"workItemId": work_item_id, "verification": [
        "cargo fmt --all -- --check",
        "cargo clippy --locked --workspace --all-targets --all-features -- -D warnings",
        "cargo test --locked --workspace",
    ]}), encoding="utf-8"
)
contract = json.loads((repository / ".ai/work-items/active" / f"{work_item_id}.contract.json").read_text(encoding="utf-8"))
contract_digest = "sha256:" + hashlib.sha256(
    json.dumps(contract, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
).hexdigest()
(repository.parent / "contract-digest").write_text(contract_digest, encoding="utf-8")
receipt = {
    "workItemId": work_item_id,
    "passed": True,
    "runtimeDigest": runtime_digest,
    "runtimeVersion": "0.2.113",
    "repositoryId": repository_id,
    "nodesPlanned": len(packages) * 2 + 1,
    "nodesExecuted": len(packages) * 2 + 1,
    "nodesReused": 0,
    "results": [
        {"nodeId": f"project-command-2-package-{package}", "passed": True,
         "reused": False, "action": "execute", "satisfiedBy": "execution"}
        for package in packages
    ] + [{"nodeId": "project-command-0", "passed": True}]
    + [{"nodeId": node, "passed": True} for node in lint_nodes],
    "executionRecords": [
        {"nodeId": node, "commandDigest": command_digest, "spawned": True, "passed": True, "exitCode": 0, "stdout": "", "stderr": "", "stdoutTruncated": False, "stderrTruncated": False}
        for node in test_nodes
    ],
    "planReceipt": {
        "workItemId": work_item_id,
        "repositoryId": repository_id,
        "repositorySnapshotDigest": snapshot_digest,
        "executedNodes": test_nodes + ["project-command-0"] + lint_nodes,
        "reusedNodes": [],
        "coverageManifest": {
            "sourceProgram": "cargo",
            "sourceArgs": ["test", "--locked", "--workspace"],
            "workspaceMembers": packages,
            "nodeIds": test_nodes,
            "commandDigests": [command_digest for _ in packages],
        },
    },
}
evidence = {
    "workItemId": work_item_id,
    "passed": True,
    "runtimeDigest": runtime_digest,
    "runtimeVersion": "0.2.113",
    "contractDigest": contract_digest,
    "repositoryId": repository_id,
    "repositorySnapshotDigest": snapshot_digest,
    "receipt": receipt,
}
evidence_bytes = json.dumps(evidence).encode("utf-8")
receipt_path.write_bytes(evidence_bytes)
(repository / ".ai/evidence" / f"{work_item_id}.verification.json").write_bytes(evidence_bytes)
PY
refresh_hosted_orchestration() {
  local verification_state=${1:-passed}
  python3 - "$tmp/runtime-bin" "$tmp/hosted-verification.json" "$tmp/hosted-repository" \
    "$tmp/hosted-orchestration.json" "$verification_state" "$root/tests/ci/run_workspace_package_tests.sh" <<'PY'
import ast
import hashlib
import json
import os
import re
import stat
import subprocess
import sys
from pathlib import Path

runtime = Path(sys.argv[1])
receipt = Path(sys.argv[2])
repository = Path(sys.argv[3])
orchestration = Path(sys.argv[4])
state = sys.argv[5]
source = Path(sys.argv[6]).read_text().split("def cargo_reuse_context(", 1)[1].split("\ndef captured_target_coverage(", 1)[0]
function = ast.parse("def cargo_reuse_context(" + source).body[0]
namespace = dict(hashlib=hashlib, os=os, re=re, stat=stat, subprocess=subprocess, Path=Path)
exec(compile(ast.Module(body=[function], type_ignores=[]), "actual-canonical-cargo-context", "exec"), namespace)
context = namespace["cargo_reuse_context"](repository, Path(sys.argv[6]).parent / "run_process_observer_test_runner.sh")
formal_bytes = receipt.read_bytes()
formal = json.loads(formal_bytes)
plan = formal["receipt"].get("planReceipt") if isinstance(formal.get("receipt"), dict) else None
orchestration.write_text(
    json.dumps({
        "schemaVersion": 1,
        "workItemId": formal["workItemId"],
        "runtimeDigest": "sha256:" + hashlib.sha256(runtime.read_bytes()).hexdigest(),
        "formalReceiptDigest": "sha256:" + hashlib.sha256(formal_bytes).hexdigest(),
        "executionRepository": str(repository.resolve()),
        "verificationState": state,
        "executionOutput": {"cargoContext": context} if isinstance(plan, dict) and plan.get("coverageManifest") is not None else None,
    }),
    encoding="utf-8",
)
PY
}
refresh_hosted_orchestration passed
export AI_COCKPIT_VERIFICATION_ORCHESTRATION="$tmp/hosted-orchestration.json"
PACKAGE_LOG="$tmp/hosted-packages.log" \
FAKE_RUNTIME_LOG="$tmp/runtime-status.log" \
FAKE_CURRENT_CONTRACT_DIGEST="$(<"$tmp/contract-digest")" \
FAKE_CURRENT_SNAPSHOT_DIGEST="sha256:$(printf '2%.0s' {1..64})" \
AI_COCKPIT_VERIFICATION_RECEIPT="$tmp/hosted-verification.json" \
AI_COCKPIT_RUNTIME_BIN="$tmp/runtime-bin" \
AI_COCKPIT_VERIFICATION_REPOSITORY="$tmp/hosted-repository" \
  "$root/tests/ci/run_workspace_package_tests.sh" \
    --metadata "$tmp/metadata.json" --cargo "$tmp/fake-cargo" --report "$tmp/hosted-report.json"
jq -e '.state == "passed" and .planned == ["package-a", "package-b"] and .executed == .planned' \
  "$tmp/hosted-report.json" >/dev/null
jq -e '.verificationReceiptState == "passed" and .executedByHostedRuntime == ["package-a", "package-b"] and .executedByCoverageRunner == []' \
  "$tmp/hosted-report.json" >/dev/null
test "$(wc -l <"$tmp/runtime-status.log" | tr -d ' ')" = 1 || {
  printf 'coverage did not ask the candidate Runtime to verify receipt freshness\n' >&2
  exit 1
}
test ! -s "$tmp/hosted-packages.log" || {
  printf 'coverage re-ran package checks already present in the matching formal receipt\n' >&2
  exit 1
}

# Canonical manifests cannot bypass host-target or runner checks either.
canonical_context_cases=0
for canonical_context in nonhost missing_context foreign_runner; do
  refresh_hosted_orchestration passed
  if [[ "$canonical_context" == missing_context ]]; then
    jq 'del(.executionOutput)' "$tmp/hosted-orchestration.json" >"$tmp/no-context.json"
    mv "$tmp/no-context.json" "$tmp/hosted-orchestration.json"
  fi
  result=0
  selected_target=$host_target
  selected_runner=$root/tests/ci/run_process_observer_test_runner.sh
  if [[ "$canonical_context" == nonhost ]]; then selected_target=aarch64-unknown-linux-gnu; fi
  if [[ "$canonical_context" == foreign_runner ]]; then selected_runner=foreign-runner; fi
  env "CARGO_BUILD_TARGET=$selected_target" "$runner_variable=$selected_runner" \
  PACKAGE_LOG="$tmp/canonical-$canonical_context-packages.log" \
  FAKE_RUNTIME_LOG="$tmp/runtime-status.log" \
  FAKE_CURRENT_CONTRACT_DIGEST="$(<"$tmp/contract-digest")" \
  FAKE_CURRENT_SNAPSHOT_DIGEST="sha256:$(printf '2%.0s' {1..64})" \
  AI_COCKPIT_VERIFICATION_RECEIPT="$tmp/hosted-verification.json" \
  AI_COCKPIT_RUNTIME_BIN="$tmp/runtime-bin" \
  AI_COCKPIT_VERIFICATION_REPOSITORY="$tmp/hosted-repository" \
    "$root/tests/ci/run_workspace_package_tests.sh" --metadata "$tmp/metadata.json" \
      --cargo "$tmp/fake-cargo" --report "$tmp/canonical-$canonical_context-report.json" >"$tmp/canonical-$canonical_context.log" 2>&1 || result=$?
  if [[ "$canonical_context" == foreign_runner ]]; then
    test "$result" != 0
    jq -e '.state == "failed" and .failurePhase == "runner_setup" and .executed == []' "$tmp/canonical-$canonical_context-report.json" >/dev/null
  else
    test "$result" = 0
    diff -u <(printf '%s\n' package-a package-b) "$tmp/canonical-$canonical_context-packages.log"
    jq -e '.state == "passed" and .executedByHostedRuntime == [] and .executedByCoverageRunner == .planned' "$tmp/canonical-$canonical_context-report.json" >/dev/null
  fi
  canonical_context_cases=$((canonical_context_cases + 1))
done
printf 'canonical manifest Cargo context regressions passed: %s\n' "$canonical_context_cases"
refresh_hosted_orchestration passed

# When Runtime reused only one package node, the coverage consumer runs just
# that node and retains the current hosted execution for the other package.
python3 - "$tmp/hosted-verification.json" "$tmp/hosted-repository/.ai/evidence/WI-HOSTED-RECEIPT.verification.json" <<'PY'
import json
import sys
from pathlib import Path

receipt_path, evidence_path = map(Path, sys.argv[1:])
formal = json.loads(receipt_path.read_text(encoding="utf-8"))
receipt = formal["receipt"]
reused_id = "project-command-2-package-package-b"
receipt["nodesReused"] = 1
receipt["nodesExecuted"] = len(receipt["results"]) - 1
receipt["planReceipt"]["reusedNodes"] = [reused_id]
receipt["planReceipt"]["executedNodes"] = [
    result["nodeId"] for result in receipt["results"] if result["nodeId"] != reused_id
]
for result in receipt["results"]:
    result["reused"] = result["nodeId"] == reused_id
    if result["nodeId"] == reused_id:
        result["action"] = "reuse"
        result["satisfiedBy"] = "reused_receipt"
receipt["executionRecords"] = [
    record for record in receipt["executionRecords"] if record["nodeId"] != reused_id
]
encoded = json.dumps(formal).encode("utf-8")
receipt_path.write_bytes(encoded)
evidence_path.write_bytes(encoded)
PY
refresh_hosted_orchestration passed
PACKAGE_LOG="$tmp/partial-hosted-packages.log" \
FAKE_RUNTIME_LOG="$tmp/runtime-status.log" \
FAKE_CURRENT_CONTRACT_DIGEST="$(<"$tmp/contract-digest")" \
FAKE_CURRENT_SNAPSHOT_DIGEST="sha256:$(printf '2%.0s' {1..64})" \
AI_COCKPIT_VERIFICATION_RECEIPT="$tmp/hosted-verification.json" \
AI_COCKPIT_RUNTIME_BIN="$tmp/runtime-bin" \
AI_COCKPIT_VERIFICATION_REPOSITORY="$tmp/hosted-repository" \
  "$root/tests/ci/run_workspace_package_tests.sh" \
    --metadata "$tmp/metadata.json" --cargo "$tmp/fake-cargo" --report "$tmp/partial-hosted-report.json"
diff -u <(printf '%s\n' package-b) "$tmp/partial-hosted-packages.log"
jq -e '.state == "passed" and .executed == ["package-a", "package-b"] and .executedByHostedRuntime == ["package-a"] and .executedByCoverageRunner == ["package-b"]' \
  "$tmp/partial-hosted-report.json" >/dev/null

# A fresh receipt from before this hosted run is reusable Runtime evidence, but
# it is not proof that this hosted run executed the package checks.  The
# coverage consumer must run the missing hosted package checks in that case.
refresh_hosted_orchestration reused
PACKAGE_LOG="$tmp/reused-hosted-packages.log" \
FAKE_RUNTIME_LOG="$tmp/runtime-status.log" \
FAKE_CURRENT_CONTRACT_DIGEST="$(<"$tmp/contract-digest")" \
FAKE_CURRENT_SNAPSHOT_DIGEST="sha256:$(printf '2%.0s' {1..64})" \
AI_COCKPIT_VERIFICATION_RECEIPT="$tmp/hosted-verification.json" \
AI_COCKPIT_VERIFICATION_ORCHESTRATION="$tmp/hosted-orchestration.json" \
AI_COCKPIT_RUNTIME_BIN="$tmp/runtime-bin" \
AI_COCKPIT_VERIFICATION_REPOSITORY="$tmp/hosted-repository" \
  "$root/tests/ci/run_workspace_package_tests.sh" \
    --metadata "$tmp/metadata.json" --cargo "$tmp/fake-cargo" --report "$tmp/reused-hosted-report.json"
diff -u <(printf '%s\n' package-a package-b | sort) <(sort "$tmp/reused-hosted-packages.log")
jq -e '.state == "passed" and .executed == ["package-a", "package-b"]' \
  "$tmp/reused-hosted-report.json" >/dev/null
jq -e '.verificationReceiptState == "reused" and .executedByHostedRuntime == [] and .executedByCoverageRunner == ["package-a", "package-b"]' \
  "$tmp/reused-hosted-report.json" >/dev/null

cp "$tmp/hosted-verification.json" "$tmp/valid-hosted-verification.json"
cp "$tmp/hosted-repository/.ai/work-items/active/WI-HOSTED-RECEIPT.contract.json" "$tmp/valid-hosted-contract.json"
cp "$tmp/contract-digest" "$tmp/valid-contract-digest"
# A one-command-index legacy route must still consume the same formal
# package evidence. Swap the declared test and fmt stages without changing
# package execution or reuse semantics.
python3 - "$tmp/hosted-verification.json" \
  "$tmp/hosted-repository/.ai/evidence/WI-HOSTED-RECEIPT.verification.json" \
  "$tmp/hosted-repository/.ai/work-items/active/WI-HOSTED-RECEIPT.contract.json" \
  "$tmp/contract-digest" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

receipt_path, evidence_path, contract_path, digest_path = map(Path, sys.argv[1:])
formal = json.loads(receipt_path.read_text(encoding="utf-8"))
receipt = formal["receipt"]
contract = json.loads(contract_path.read_text(encoding="utf-8"))
contract["verification"][0], contract["verification"][2] = (
    contract["verification"][2], contract["verification"][0]
)
contract_path.write_text(json.dumps(contract), encoding="utf-8")
contract_digest = "sha256:" + hashlib.sha256(
    json.dumps(contract, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
).hexdigest()
digest_path.write_text(contract_digest, encoding="utf-8")
formal["contractDigest"] = contract_digest
def swap(node):
    if node.startswith("project-command-0"):
        return node.replace("project-command-0", "project-command-2", 1)
    if node.startswith("project-command-2"):
        return node.replace("project-command-2", "project-command-0", 1)
    return node
for result in receipt["results"]:
    result["nodeId"] = swap(result["nodeId"])
for record in receipt["executionRecords"]:
    record["nodeId"] = swap(record["nodeId"])
for name in ["executedNodes", "reusedNodes"]:
    receipt["planReceipt"][name] = [swap(node) for node in receipt["planReceipt"][name]]
receipt["planReceipt"]["coverageManifest"]["nodeIds"] = [
    swap(node) for node in receipt["planReceipt"]["coverageManifest"]["nodeIds"]
]
encoded = json.dumps(formal).encode("utf-8")
receipt_path.write_bytes(encoded)
evidence_path.write_bytes(encoded)
PY
refresh_hosted_orchestration passed
PACKAGE_LOG="$tmp/legacy-packages.log" \
FAKE_RUNTIME_LOG="$tmp/runtime-status.log" \
FAKE_CURRENT_CONTRACT_DIGEST="$(<"$tmp/contract-digest")" \
FAKE_CURRENT_SNAPSHOT_DIGEST="sha256:$(printf '2%.0s' {1..64})" \
AI_COCKPIT_VERIFICATION_RECEIPT="$tmp/hosted-verification.json" \
AI_COCKPIT_RUNTIME_BIN="$tmp/runtime-bin" \
AI_COCKPIT_VERIFICATION_REPOSITORY="$tmp/hosted-repository" \
  "$root/tests/ci/run_workspace_package_tests.sh" \
    --metadata "$tmp/metadata.json" --cargo "$tmp/fake-cargo" --report "$tmp/legacy-report.json"
diff -u <(printf '%s\n' package-b) "$tmp/legacy-packages.log"
jq -e '.state == "passed" and .executedByHostedRuntime == ["package-a"] and .executedByCoverageRunner == ["package-b"]' \
  "$tmp/legacy-report.json" >/dev/null
cp "$tmp/valid-contract-digest" "$tmp/contract-digest"
cp "$tmp/valid-hosted-verification.json" "$tmp/hosted-verification.json"
cp "$tmp/valid-hosted-verification.json" \
  "$tmp/hosted-repository/.ai/evidence/WI-HOSTED-RECEIPT.verification.json"
cp "$tmp/valid-hosted-contract.json" "$tmp/hosted-repository/.ai/work-items/active/WI-HOSTED-RECEIPT.contract.json"
refresh_hosted_orchestration passed
for tamper in clippy_relabel mixed_index forged_index wrong_digest missing_package duplicate_package extra_package filtered only_clippy duplicate_test env_duplicate_test multi_env_duplicate_test quoted_env_duplicate_test only_env_test explicit_env_duplicate_test only_explicit_env_test env_option_test nested_env_test missing_plan; do
  cp "$tmp/valid-hosted-verification.json" "$tmp/hosted-verification.json"
  cp "$tmp/valid-hosted-contract.json" "$tmp/hosted-repository/.ai/work-items/active/WI-HOSTED-RECEIPT.contract.json"
  cp "$tmp/valid-contract-digest" "$tmp/contract-digest"
  python3 - "$tamper" "$tmp/hosted-verification.json" \
    "$tmp/hosted-repository/.ai/evidence/WI-HOSTED-RECEIPT.verification.json" \
    "$tmp/hosted-repository/.ai/work-items/active/WI-HOSTED-RECEIPT.contract.json" \
    "$tmp/contract-digest" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

case = sys.argv[1]
receipt_path, evidence_path, contract_path, digest_path = map(Path, sys.argv[2:])
formal = json.loads(receipt_path.read_text(encoding="utf-8"))
receipt = formal["receipt"]
coverage = receipt["planReceipt"]["coverageManifest"]
if case == "clippy_relabel":
    coverage["nodeIds"] = [node.replace("command-2", "command-1") for node in coverage["nodeIds"]]
elif case == "mixed_index":
    coverage["nodeIds"][0] = coverage["nodeIds"][0].replace("command-2", "command-1")
elif case == "forged_index":
    coverage["nodeIds"] = [node.replace("command-2", "command-3") for node in coverage["nodeIds"]]
elif case == "wrong_digest":
    coverage["commandDigests"][0] = "sha256:" + "f" * 64
elif case == "missing_package":
    coverage["nodeIds"].pop()
elif case == "duplicate_package":
    coverage["nodeIds"][1] = coverage["nodeIds"][0]
elif case == "extra_package":
    extra_node = "project-command-2-package-package-c"
    receipt["results"].append({"nodeId": extra_node, "passed": True})
    receipt["planReceipt"]["executedNodes"].append(extra_node)
    receipt["nodesPlanned"] += 1
    receipt["nodesExecuted"] += 1
elif case == "filtered":
    coverage["sourceArgs"] += ["--", "package_a_only"]
elif case == "only_clippy":
    coverage["sourceArgs"][0] = "clippy"
elif case in {"duplicate_test", "env_duplicate_test", "multi_env_duplicate_test", "quoted_env_duplicate_test", "only_env_test", "explicit_env_duplicate_test", "only_explicit_env_test", "env_option_test", "nested_env_test"}:
    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    declaration = {
        "duplicate_test": "cargo test --locked --workspace",
        "env_duplicate_test": "FOO=1 cargo test --locked --workspace",
        "multi_env_duplicate_test": "FOO=1 BAR=two cargo test --locked --workspace",
        "quoted_env_duplicate_test": 'FOO=1 BAR="two words" cargo test --locked --workspace',
        "only_env_test": "FOO=1 cargo test --locked --workspace",
        "explicit_env_duplicate_test": "env FOO=1 cargo test --locked --workspace",
        "only_explicit_env_test": "env FOO=1 cargo test --locked --workspace",
        "env_option_test": "env -i FOO=1 cargo test --locked --workspace",
        "nested_env_test": "env FOO=1 env BAR=2 cargo test --locked --workspace",
    }[case]
    if case in {"only_env_test", "only_explicit_env_test"}:
        contract["verification"][2] = declaration
    else:
        contract["verification"].append(declaration)
    contract_path.write_text(json.dumps(contract), encoding="utf-8")
    contract_digest = "sha256:" + hashlib.sha256(
        json.dumps(contract, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    ).hexdigest()
    digest_path.write_text(contract_digest, encoding="utf-8")
    formal["contractDigest"] = contract_digest
elif case == "missing_plan":
    receipt.pop("planReceipt")
else:
    raise SystemExit(f"unknown case: {case}")
encoded = json.dumps(formal).encode("utf-8")
receipt_path.write_bytes(encoded)
evidence_path.write_bytes(encoded)
PY
  refresh_hosted_orchestration passed
  if PACKAGE_LOG="$tmp/tamper-$tamper-package-processes.log" \
    FAKE_RUNTIME_LOG="$tmp/runtime-status.log" \
    FAKE_CURRENT_CONTRACT_DIGEST="$(<"$tmp/contract-digest")" \
    FAKE_CURRENT_SNAPSHOT_DIGEST="sha256:$(printf '2%.0s' {1..64})" \
    AI_COCKPIT_VERIFICATION_RECEIPT="$tmp/hosted-verification.json" \
    AI_COCKPIT_RUNTIME_BIN="$tmp/runtime-bin" \
    AI_COCKPIT_VERIFICATION_REPOSITORY="$tmp/hosted-repository" \
    "$root/tests/ci/run_workspace_package_tests.sh" \
      --metadata "$tmp/metadata.json" --cargo "$tmp/fake-cargo" --report "$tmp/tamper-$tamper.json" \
      >/dev/null 2>&1; then
    printf 'workspace coverage accepted %s tampering\n' "$tamper" >&2
    exit 1
  fi
  if ! jq -e '.state == "failed" and .failurePhase == "hosted_verification_receipt" and .executed == []' \
    "$tmp/tamper-$tamper.json" >/dev/null; then
    printf 'workspace coverage did not fail closed before package execution for %s\n' "$tamper" >&2
    jq '{state, failurePhase, executed, failureDiagnosticTail}' "$tmp/tamper-$tamper.json" >&2
    exit 1
  fi
  if [[ -e "$tmp/tamper-$tamper-package-processes.log" ]]; then
    printf 'workspace coverage spawned package cargo for %s\n' "$tamper" >&2
    exit 1
  fi
  if [[ "$tamper" == clippy_relabel || "$tamper" == duplicate_test ]]; then
    jq -e '.failureDiagnosticTail | contains("unique matching Contract declaration")' \
      "$tmp/tamper-$tamper.json" >/dev/null
  fi
  if [[ "$tamper" == env_duplicate_test || "$tamper" == multi_env_duplicate_test || "$tamper" == quoted_env_duplicate_test || "$tamper" == only_env_test || "$tamper" == explicit_env_duplicate_test || "$tamper" == only_explicit_env_test ]]; then
    jq -e '.failureDiagnosticTail | contains("unsupported environment-wrapped Cargo test")' \
      "$tmp/tamper-$tamper.json" >/dev/null
  fi
  if [[ "$tamper" == env_option_test || "$tamper" == nested_env_test ]]; then
    jq -e '.failureDiagnosticTail | contains("unsupported env command syntax")' \
      "$tmp/tamper-$tamper.json" >/dev/null
  fi
done
cp "$tmp/valid-hosted-verification.json" "$tmp/hosted-verification.json"
cp "$tmp/valid-hosted-verification.json" \
  "$tmp/hosted-repository/.ai/evidence/WI-HOSTED-RECEIPT.verification.json"
cp "$tmp/valid-hosted-contract.json" "$tmp/hosted-repository/.ai/work-items/active/WI-HOSTED-RECEIPT.contract.json"
python3 - "$tmp/hosted-verification.json" \
  "$tmp/hosted-repository/.ai/evidence/WI-HOSTED-RECEIPT.verification.json" \
  "$tmp/hosted-repository/.ai/work-items/active/WI-HOSTED-RECEIPT.contract.json" \
  "$tmp/contract-digest" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

receipt_path, evidence_path, contract_path, digest_path = map(Path, sys.argv[1:])
formal = json.loads(receipt_path.read_text(encoding="utf-8"))
contract = json.loads(contract_path.read_text(encoding="utf-8"))
contract["verification"].append("FOO=1 cargo clippy --locked --workspace")
contract["verification"].append("env FOO=1 cargo clippy --locked --workspace")
contract_path.write_text(json.dumps(contract), encoding="utf-8")
digest = "sha256:" + hashlib.sha256(
    json.dumps(contract, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
).hexdigest()
digest_path.write_text(digest, encoding="utf-8")
formal["contractDigest"] = digest
encoded = json.dumps(formal).encode("utf-8")
receipt_path.write_bytes(encoded)
evidence_path.write_bytes(encoded)
PY
refresh_hosted_orchestration passed
PACKAGE_LOG="$tmp/non-cargo-env-packages.log" \
FAKE_RUNTIME_LOG="$tmp/runtime-status.log" \
  FAKE_CURRENT_CONTRACT_DIGEST="$(<"$tmp/contract-digest")" \
  FAKE_CURRENT_SNAPSHOT_DIGEST="sha256:$(printf '2%.0s' {1..64})" \
  AI_COCKPIT_VERIFICATION_RECEIPT="$tmp/hosted-verification.json" \
  AI_COCKPIT_RUNTIME_BIN="$tmp/runtime-bin" \
  AI_COCKPIT_VERIFICATION_REPOSITORY="$tmp/hosted-repository" \
  "$root/tests/ci/run_workspace_package_tests.sh" \
    --metadata "$tmp/metadata.json" --cargo "$tmp/fake-cargo" --report "$tmp/non-cargo-env-report.json"
jq -e '.state == "passed" and .executedByCoverageRunner == ["package-b"]' \
  "$tmp/non-cargo-env-report.json" >/dev/null
cp "$tmp/valid-hosted-verification.json" "$tmp/hosted-verification.json"
cp "$tmp/valid-hosted-verification.json" \
  "$tmp/hosted-repository/.ai/evidence/WI-HOSTED-RECEIPT.verification.json"
cp "$tmp/valid-hosted-contract.json" "$tmp/hosted-repository/.ai/work-items/active/WI-HOSTED-RECEIPT.contract.json"
cp "$tmp/valid-contract-digest" "$tmp/contract-digest"
refresh_hosted_orchestration passed
python3 - "$tmp/hosted-verification.json" "$tmp/hosted-repository/.ai/evidence/WI-HOSTED-RECEIPT.verification.json" <<'PY'
import json
import sys
from pathlib import Path

receipt_path, evidence_path = map(Path, sys.argv[1:])
evidence = json.loads(receipt_path.read_text(encoding="utf-8"))
evidence["contractDigest"] = "sha256:" + "4" * 64
encoded = json.dumps(evidence).encode("utf-8")
receipt_path.write_bytes(encoded)
evidence_path.write_bytes(encoded)
PY
refresh_hosted_orchestration passed
if FAKE_RUNTIME_LOG="$tmp/runtime-status.log" \
  FAKE_CURRENT_CONTRACT_DIGEST="$(<"$tmp/contract-digest")" \
  FAKE_CURRENT_SNAPSHOT_DIGEST="sha256:$(printf '2%.0s' {1..64})" \
  AI_COCKPIT_VERIFICATION_RECEIPT="$tmp/hosted-verification.json" \
  AI_COCKPIT_RUNTIME_BIN="$tmp/runtime-bin" \
  AI_COCKPIT_VERIFICATION_REPOSITORY="$tmp/hosted-repository" \
  "$root/tests/ci/run_workspace_package_tests.sh" \
    --metadata "$tmp/metadata.json" --cargo "$tmp/fake-cargo" --report "$tmp/stale-contract-report.json" \
    >/dev/null 2>&1; then
  printf 'workspace coverage accepted a receipt bound to a stale Contract digest\n' >&2
  exit 1
fi
jq -e '.state == "failed" and .failurePhase == "hosted_verification_receipt" and (.failureDiagnosticTail | contains("current Runtime Contract"))' \
  "$tmp/stale-contract-report.json" >/dev/null
cp "$tmp/valid-hosted-verification.json" "$tmp/hosted-verification.json"
cp "$tmp/valid-hosted-verification.json" \
  "$tmp/hosted-repository/.ai/evidence/WI-HOSTED-RECEIPT.verification.json"
refresh_hosted_orchestration passed

python3 - "$tmp/hosted-verification.json" "$tmp/hosted-repository/.ai/evidence/WI-HOSTED-RECEIPT.verification.json" <<'PY'
import json
import sys
from pathlib import Path

receipt_path, evidence_path = map(Path, sys.argv[1:])
evidence = json.loads(receipt_path.read_text(encoding="utf-8"))
stale_snapshot = "sha256:" + "4" * 64
evidence["repositorySnapshotDigest"] = stale_snapshot
evidence["receipt"]["planReceipt"]["repositorySnapshotDigest"] = stale_snapshot
encoded = json.dumps(evidence).encode("utf-8")
receipt_path.write_bytes(encoded)
evidence_path.write_bytes(encoded)
PY
refresh_hosted_orchestration passed
if FAKE_RUNTIME_LOG="$tmp/runtime-status.log" \
  FAKE_CURRENT_CONTRACT_DIGEST="$(<"$tmp/contract-digest")" \
  FAKE_CURRENT_SNAPSHOT_DIGEST="sha256:$(printf '2%.0s' {1..64})" \
  AI_COCKPIT_VERIFICATION_RECEIPT="$tmp/hosted-verification.json" \
  AI_COCKPIT_RUNTIME_BIN="$tmp/runtime-bin" \
  AI_COCKPIT_VERIFICATION_REPOSITORY="$tmp/hosted-repository" \
  "$root/tests/ci/run_workspace_package_tests.sh" \
    --metadata "$tmp/metadata.json" --cargo "$tmp/fake-cargo" --report "$tmp/stale-snapshot-report.json" \
    >/dev/null 2>&1; then
  printf 'workspace coverage accepted a receipt bound to a stale source snapshot\n' >&2
  exit 1
fi
jq -e '.state == "failed" and .failurePhase == "hosted_verification_receipt" and (.failureDiagnosticTail | contains("current Runtime source snapshot"))' \
  "$tmp/stale-snapshot-report.json" >/dev/null
cp "$tmp/valid-hosted-verification.json" "$tmp/hosted-verification.json"
cp "$tmp/valid-hosted-verification.json" \
  "$tmp/hosted-repository/.ai/evidence/WI-HOSTED-RECEIPT.verification.json"
refresh_hosted_orchestration passed

python3 - "$tmp/hosted-verification.json" "$tmp/hosted-repository/.ai/evidence/WI-HOSTED-RECEIPT.verification.json" <<'PY'
import json
import sys
from pathlib import Path

receipt_path, evidence_path = map(Path, sys.argv[1:])
evidence = json.loads(evidence_path.read_text(encoding="utf-8"))
evidence["receipt"]["annotation"] = "same identity, different bytes"
evidence_path.write_text(json.dumps(evidence, indent=2), encoding="utf-8")
PY
refresh_hosted_orchestration passed
if AI_COCKPIT_VERIFICATION_RECEIPT="$tmp/hosted-verification.json" \
  AI_COCKPIT_RUNTIME_BIN="$tmp/runtime-bin" \
  AI_COCKPIT_VERIFICATION_REPOSITORY="$tmp/hosted-repository" \
  "$root/tests/ci/run_workspace_package_tests.sh" \
    --metadata "$tmp/metadata.json" --cargo "$tmp/fake-cargo" --report "$tmp/divergent-evidence-report.json" \
    >/dev/null 2>&1; then
  printf 'workspace coverage accepted non-identical Runtime and coverage evidence bytes\n' >&2
  exit 1
fi
jq -e '.state == "failed" and .failurePhase == "hosted_verification_receipt" and .executed == []' \
  "$tmp/divergent-evidence-report.json" >/dev/null
cp "$tmp/hosted-verification.json" \
  "$tmp/hosted-repository/.ai/evidence/WI-HOSTED-RECEIPT.verification.json"
python3 - "$tmp/hosted-verification.json" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
receipt = json.loads(path.read_text(encoding="utf-8"))
receipt["runtimeDigest"] = "sha256:" + "f" * 64
path.write_text(json.dumps(receipt), encoding="utf-8")
PY
refresh_hosted_orchestration passed
if AI_COCKPIT_VERIFICATION_RECEIPT="$tmp/hosted-verification.json" \
  AI_COCKPIT_RUNTIME_BIN="$tmp/runtime-bin" \
  AI_COCKPIT_VERIFICATION_REPOSITORY="$tmp/hosted-repository" \
  "$root/tests/ci/run_workspace_package_tests.sh" \
    --metadata "$tmp/metadata.json" --cargo "$tmp/fake-cargo" --report "$tmp/invalid-hosted-report.json" \
    >/dev/null 2>&1; then
  printf 'workspace coverage accepted a verification receipt bound to another executable\n' >&2
  exit 1
fi
jq -e '.state == "failed" and .failurePhase == "hosted_verification_receipt" and .executed == []' \
  "$tmp/invalid-hosted-report.json" >/dev/null

# A serial libtest scheduling option does not filter the workspace. Explicit
# package/target regressions can accompany it, but cannot supply a missing
# workspace result. Exercise the real consumer, not a copy of its predicate.
serial_receipt_case_count=0
for coverage_case in serial_equal serial_separate targeted_serial targeted_default \
  filter_tail ignored_tail skipped_tail parallel_threads zero_threads duplicate_threads \
  duplicate_separator unknown_cargo_option unknown_target_option foreign_target_package \
  targeted_filter targeted_environment targeted_only missing_package_result failed_package_result \
  failed_targeted_result missing_targeted_result foreign_snapshot foreign_contract \
  native_warning native_bad_hex native_truncated native_missing_capture; do
  cp "$tmp/valid-hosted-verification.json" "$tmp/hosted-verification.json"
  cp "$tmp/valid-hosted-contract.json" "$tmp/hosted-repository/.ai/work-items/active/WI-HOSTED-RECEIPT.contract.json"
  python3 - "$coverage_case" "$tmp/hosted-verification.json" \
    "$tmp/hosted-repository/.ai/evidence/WI-HOSTED-RECEIPT.verification.json" \
    "$tmp/hosted-repository/.ai/work-items/active/WI-HOSTED-RECEIPT.contract.json" \
    "$tmp/contract-digest" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

case = sys.argv[1]
receipt_path, evidence_path, contract_path, digest_path = map(Path, sys.argv[2:])
formal = json.loads(receipt_path.read_text(encoding="utf-8"))
receipt = formal["receipt"]
plan = receipt["planReceipt"]
coverage = plan["coverageManifest"]
contract = json.loads(contract_path.read_text(encoding="utf-8"))
# The saved legacy fixture intentionally reuses package-b. These new cases
# model this run executing every package; leave the original reuse cases alone.
receipt["nodesReused"] = 0
receipt["nodesExecuted"] = receipt["nodesPlanned"]
plan["reusedNodes"] = []
plan["executedNodes"] = [result["nodeId"] for result in receipt["results"]]
for result in receipt["results"]:
    if result["nodeId"] == "project-command-2-package-package-b":
        result.update(reused=False, action="execute", satisfiedBy="execution")
receipt["executionRecords"].append({
    "nodeId": "project-command-2-package-package-b", "commandDigest": "sha256:" + "4" * 64,
    "spawned": True, "passed": True, "exitCode": 0, "stdout": "", "stderr": "", "stdoutTruncated": False, "stderrTruncated": False,
})
serial_tail = ["--", "--test-threads=1"]
if case == "serial_separate":
    serial_tail = ["--", "--test-threads", "1"]
elif case == "targeted_default":
    serial_tail = []
elif case == "filter_tail":
    serial_tail = ["--", "one_test_only"]
elif case == "ignored_tail":
    serial_tail = ["--", "--test-threads=1", "--ignored"]
elif case == "skipped_tail":
    serial_tail = ["--", "--test-threads=1", "--skip", "important_test"]
elif case == "parallel_threads":
    serial_tail = ["--", "--test-threads=2"]
elif case == "zero_threads":
    serial_tail = ["--", "--test-threads=0"]
elif case == "duplicate_threads":
    serial_tail = ["--", "--test-threads=1", "--test-threads=1"]
elif case == "duplicate_separator":
    serial_tail = ["--", "--", "--test-threads=1"]
coverage["sourceArgs"] = ["test", "--locked", "--workspace"] + serial_tail
if case == "unknown_cargo_option":
    coverage["sourceArgs"].insert(1, "--unrecognized-option")
contract["verification"][2] = "cargo " + " ".join(coverage["sourceArgs"])

if case not in {"serial_equal", "serial_separate"}:
    targeted = [
        "cargo test --locked -p package-a --test acceptance -- --test-threads=1",
        "cargo test --locked -p package-b --test unit --test locks --test journal -- --test-threads=1",
        "cargo test --locked --package package-a --test cli -- --test-threads 1",
        "cargo test --locked -p package-b --test rpc -- --test-threads=1",
    ]
    if case == "unknown_target_option":
        targeted[-1] = "cargo test --locked -p package-b --test rpc --unrecognized-option"
    elif case == "foreign_target_package":
        targeted[-1] = "cargo test --locked -p not-a-workspace-member --test rpc"
    elif case == "targeted_filter":
        targeted[-1] += " one_test_only"
    elif case == "targeted_environment":
        targeted[-1] = "FOO=1 " + targeted[-1]
    for command in targeted:
        index = len(contract["verification"])
        contract["verification"].append(command)
        node = f"project-command-{index}"
        plan["executedNodes"].append(node)
        receipt["results"].append({"nodeId": node, "passed": True,
                                   "reused": False, "action": "execute", "satisfiedBy": "execution"})
        receipt["executionRecords"].append({"nodeId": node, "commandDigest": "sha256:" + "5" * 64,
                                            "spawned": True, "passed": True, "exitCode": 0, "stdout": "", "stderr": "", "stdoutTruncated": False, "stderrTruncated": False})
        receipt["nodesPlanned"] += 1
        receipt["nodesExecuted"] += 1
    if case == "targeted_only":
        contract["verification"][2] = targeted[0]
    elif case in {"missing_package_result", "missing_targeted_result"}:
        missing = "project-command-2-package-package-b" if case == "missing_package_result" else "project-command-6"
        receipt["results"] = [r for r in receipt["results"] if r["nodeId"] != missing]
        receipt["executionRecords"] = [r for r in receipt["executionRecords"] if r["nodeId"] != missing]
        plan["executedNodes"].remove(missing)
        receipt["nodesPlanned"] -= 1
        receipt["nodesExecuted"] -= 1
    elif case in {"failed_package_result", "failed_targeted_result"}:
        failed = "project-command-2-package-package-b" if case == "failed_package_result" else "project-command-6"
        for result in receipt["results"]:
            if result["nodeId"] == failed:
                result["passed"] = False
        for record in receipt["executionRecords"]:
            if record["nodeId"] == failed:
                record.update(passed=False, exitCode=17)
contract_path.write_text(json.dumps(contract), encoding="utf-8")
contract_digest = "sha256:" + hashlib.sha256(
    json.dumps(contract, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
).hexdigest()
digest_path.write_text(contract_digest, encoding="utf-8")
formal["contractDigest"] = contract_digest
if case == "foreign_contract":
    formal["contractDigest"] = "sha256:" + "f" * 64
elif case == "foreign_snapshot":
    formal["repositorySnapshotDigest"] = plan["repositorySnapshotDigest"] = "sha256:" + "f" * 64
if case.startswith("native_"):
    target_record = next(r for r in receipt["executionRecords"] if r["nodeId"] == "project-command-2-package-package-a")
    if case == "native_warning":
        target_record["stdout"] = b"warning: actual tool diagnostic with successful exit\n".hex()
    elif case == "native_bad_hex":
        target_record["stdout"] = "not-hex"
    elif case == "native_truncated":
        target_record["stdoutTruncated"] = True
    elif case == "native_missing_capture":
        for key in ["stdout", "stderr", "stdoutTruncated", "stderrTruncated"]:
            target_record.pop(key, None)
encoded = json.dumps(formal).encode("utf-8")
receipt_path.write_bytes(encoded)
evidence_path.write_bytes(encoded)
PY
  refresh_hosted_orchestration passed
  coverage_case_exit=0
  PACKAGE_LOG="$tmp/serial-$coverage_case-package-processes.log" \
  FAKE_RUNTIME_LOG="$tmp/runtime-status.log" \
  FAKE_CURRENT_CONTRACT_DIGEST="$(<"$tmp/contract-digest")" \
  FAKE_CURRENT_SNAPSHOT_DIGEST="sha256:$(printf '2%.0s' {1..64})" \
  AI_COCKPIT_VERIFICATION_RECEIPT="$tmp/hosted-verification.json" \
  AI_COCKPIT_RUNTIME_BIN="$tmp/runtime-bin" \
  AI_COCKPIT_VERIFICATION_REPOSITORY="$tmp/hosted-repository" \
    "$root/tests/ci/run_workspace_package_tests.sh" \
      --metadata "$tmp/metadata.json" --cargo "$tmp/fake-cargo" \
      --report "$tmp/serial-$coverage_case-report.json" \
      >"$tmp/serial-$coverage_case.stdout" 2>"$tmp/serial-$coverage_case.stderr" || coverage_case_exit=$?
  case "$coverage_case" in
    serial_equal|serial_separate|targeted_serial|targeted_default)
      if [[ "$coverage_case_exit" != 0 ]]; then
        printf 'workspace coverage rejected valid unfiltered case %s\n' "$coverage_case" >&2
        cat "$tmp/serial-$coverage_case.stderr" >&2
        jq '{state, failurePhase, failureDiagnosticTail}' "$tmp/serial-$coverage_case-report.json" >&2
        exit 1
      fi
      jq -e '.state == "passed" and .planned == ["package-a", "package-b"] and .executed == .planned and .executedByHostedRuntime == .planned and .executedByCoverageRunner == []' \
        "$tmp/serial-$coverage_case-report.json" >/dev/null || {
        printf 'workspace coverage returned inconsistent full execution for %s\n' "$coverage_case" >&2
        cat "$tmp/serial-$coverage_case-report.json" >&2
        exit 1
      }
      ;;
    native_truncated|native_missing_capture)
      test "$coverage_case_exit" == 0
      jq -e '.state == "passed" and .executedByHostedRuntime == ["package-a", "package-b"] and .executedByCoverageRunner == ["package-a"]'         "$tmp/serial-$coverage_case-report.json" >/dev/null
      diff -u <(printf '%s\n' package-a) "$tmp/serial-$coverage_case-package-processes.log"
      ;;
    *)
      if [[ "$coverage_case_exit" == 0 ]]; then
        printf 'workspace coverage accepted invalid serial/targeted case %s\n' "$coverage_case" >&2
        exit 1
      fi
      jq -e '.state == "failed" and .failurePhase == "hosted_verification_receipt" and .executed == []' \
        "$tmp/serial-$coverage_case-report.json" >/dev/null
      ;;
  esac
  if [[ "$coverage_case" != native_truncated && "$coverage_case" != native_missing_capture ]]; then
  test ! -e "$tmp/serial-$coverage_case-package-processes.log" || {
    printf 'serial receipt validation spawned unexpected package cargo for %s\n' "$coverage_case" >&2
    exit 1
  }
  fi
  serial_receipt_case_count=$((serial_receipt_case_count + 1))
done
printf 'serial/targeted receipt regression cases passed: %s\n' "$serial_receipt_case_count"

# Ordinary targeted plans have no workspace coverage manifest. They are not
# package coverage evidence: after validating their native identity, run every
# package instead of inventing a manifest from package names in commands.
cp "$tmp/valid-hosted-verification.json" "$tmp/hosted-verification.json"
cp "$tmp/valid-hosted-contract.json" "$tmp/hosted-repository/.ai/work-items/active/WI-HOSTED-RECEIPT.contract.json"
python3 - "$tmp/hosted-verification.json" \
  "$tmp/hosted-repository/.ai/evidence/WI-HOSTED-RECEIPT.verification.json" \
  "$tmp/hosted-repository/.ai/work-items/active/WI-HOSTED-RECEIPT.contract.json" \
  "$tmp/contract-digest" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

receipt_path, evidence_path, contract_path, digest_path = map(Path, sys.argv[1:])
formal = json.loads(receipt_path.read_text(encoding="utf-8"))
receipt = formal["receipt"]
plan = receipt["planReceipt"]
contract = json.loads(contract_path.read_text(encoding="utf-8"))
contract["verification"][2] = "cargo test --locked -p package-a --lib -- --test-threads=1"
old_nodes = set(plan.pop("coverageManifest")["nodeIds"])
receipt["results"] = [r for r in receipt["results"] if r["nodeId"] not in old_nodes]
receipt["executionRecords"] = [r for r in receipt["executionRecords"] if r["nodeId"] not in old_nodes]
receipt["results"].append({"nodeId": "project-command-2", "passed": True,
                           "reused": False, "action": "execute", "satisfiedBy": "execution"})
receipt["executionRecords"].append({"nodeId": "project-command-2", "commandDigest": "sha256:" + "5" * 64,
                                   "spawned": True, "passed": True, "exitCode": 0})
receipt["nodesPlanned"] = receipt["nodesExecuted"] = len(receipt["results"])
receipt["nodesReused"] = 0
plan["executedNodes"] = [r["nodeId"] for r in receipt["results"]]
plan["reusedNodes"] = []
contract_path.write_text(json.dumps(contract), encoding="utf-8")
digest = "sha256:" + hashlib.sha256(json.dumps(contract, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()).hexdigest()
digest_path.write_text(digest, encoding="utf-8")
formal["contractDigest"] = digest
raw = json.dumps(formal).encode()
receipt_path.write_bytes(raw)
evidence_path.write_bytes(raw)
PY
refresh_hosted_orchestration passed
PACKAGE_LOG="$tmp/targeted-fallback-packages.log" WORKSPACE_TEST_WORKERS=1 WORKSPACE_TEST_THREADS=1 \
FAKE_RUNTIME_LOG="$tmp/runtime-status.log" \
FAKE_CURRENT_CONTRACT_DIGEST="$(<"$tmp/contract-digest")" \
FAKE_CURRENT_SNAPSHOT_DIGEST="sha256:$(printf '2%.0s' {1..64})" \
AI_COCKPIT_VERIFICATION_RECEIPT="$tmp/hosted-verification.json" \
AI_COCKPIT_RUNTIME_BIN="$tmp/runtime-bin" \
AI_COCKPIT_VERIFICATION_REPOSITORY="$tmp/hosted-repository" \
  "$root/tests/ci/run_workspace_package_tests.sh" --metadata "$tmp/metadata.json" \
    --cargo "$tmp/fake-cargo" --report "$tmp/targeted-fallback-report.json"
diff -u <(printf '%s\n' package-a package-b) "$tmp/targeted-fallback-packages.log"
jq -e '.state == "passed" and .executed == .planned and .executedByHostedRuntime == [] and .executedByCoverageRunner == .planned and .groupTimeoutSeconds == 900 and .groupCleanupGraceSeconds == 30 and .workers == 1 and .testThreads == 1' \
  "$tmp/targeted-fallback-report.json" >/dev/null

# Same-job producer fixtures: these model captured Runtime output, not new
# native signatures. A coordinated rewrite of every hosted input is outside
# this existing trusted-producer boundary.
make_targeted_native_fixture() {
  local case_dir=$1
  mkdir -p "$case_dir/repository/.ai/work-items/active" "$case_dir/repository/.ai/evidence"
  cp "$tmp/metadata.json" "$case_dir/metadata.json"
  python3 - "$case_dir" <<'PY_NATIVE_FIXTURE'
import hashlib
import json
import sys
from pathlib import Path
case = Path(sys.argv[1])
repository = case / "repository"
wi = "WI-HOSTED-RECEIPT"
digest = lambda raw: "sha256:" + hashlib.sha256(raw).hexdigest()
repository_id = "sha256:" + "1" * 64
snapshot = "sha256:" + "2" * 64
runtime = digest(Path(case.parent / "runtime-bin").read_bytes())
commands = [["cargo", "test", "--locked", "-p", package, "--lib", "--", "--test-threads=1"]
            for package in ("package-a", "package-b")]
contract = {"workItemId": wi, "verification": [" ".join(words) for words in commands]}
raw_contract = json.dumps(contract).encode()
(repository / f".ai/work-items/active/{wi}.contract.json").write_bytes(raw_contract)
contract_digest = digest(json.dumps(contract, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode())
(case / "contract-digest").write_text(contract_digest)
ids = ["project-command-0", "project-command-1"]
records = [{"nodeId": node, "commandDigest": digest(json.dumps(words).encode()),
            "spawned": True, "passed": True, "exitCode": 0, "timedOut": False, "timeoutSeconds": 900,
            "stdout": "", "stderr": "", "stdoutTruncated": False, "stderrTruncated": False}
           for node, words in zip(ids, commands)]
results = [{"nodeId": node, "passed": True, "timedOut": False, "reused": False,
            "action": "execute", "satisfiedBy": "execution"} for node in ids]
plan = {"workItemId": wi, "repositoryId": repository_id, "repositorySnapshotDigest": snapshot,
        "baseRevision": "a" * 40, "stage": "task", "executedNodes": ids, "reusedNodes": []}
receipt = {"workItemId": wi, "passed": True, "repositoryId": repository_id,
           "runtimeVersion": "0.2.113", "runtimeDigest": runtime,
           "nodesPlanned": 2, "nodesExecuted": 2, "nodesReused": 0,
           "results": results, "executionRecords": records, "planReceipt": plan}
formal = {"workItemId": wi, "passed": True, "runtimeDigest": runtime, "runtimeVersion": "0.2.113",
          "repositoryId": repository_id, "repositorySnapshotDigest": snapshot,
          "contractDigest": contract_digest, "receipt": receipt}
raw = json.dumps(formal).encode()
(case / "hosted-runtime-verification.json").write_bytes(raw)
(repository / f".ai/evidence/{wi}.verification.json").write_bytes(raw)
output = {key: value for key, value in receipt.items() if key != "workItemId"}
output["plannedNodes"] = [{"nodeId": node, "program": words[0], "args": words[1:],
    "timeoutSeconds": 900, "identityBinding": {"repositoryId": repository_id,
    "repositorySnapshotDigest": snapshot, "runtimeDigest": runtime,
    "commandDigest": record["commandDigest"]}}
    for node, words, record in zip(ids, commands, records)]
output_raw = json.dumps(output).encode()
(case / "hosted-runtime-execution.json").write_bytes(output_raw)
context = {"repository": "example/fixture", "runId": "123456", "runAttempt": "1",
           "job": "fixture_quality", "headSha": "b" * 40, "runnerOs": "Linux"}
orchestration = {"schemaVersion": 1, "workItemId": wi, "runtimeDigest": runtime,
    "executionRepository": str(repository.resolve()), "formalReceiptDigest": digest(raw),
    "verificationState": "passed", "executionOutput": {"schemaVersion": 1,
    "source": "runtime_stdout", "file": "hosted-runtime-execution.json",
    "sha256": digest(output_raw), "providerContext": context}}
(case / "hosted-runtime-orchestration.json").write_text(json.dumps(orchestration))
PY_NATIVE_FIXTURE
}
run_targeted_native_fixture() {
  local case_dir=$1
  PACKAGE_LOG="$case_dir/packages.log" WORKSPACE_TEST_WORKERS=1 WORKSPACE_TEST_THREADS=1 \
  FAKE_RUNTIME_LOG="$case_dir/status.log" \
  FAKE_CURRENT_CONTRACT_DIGEST="$(<"$case_dir/contract-digest")" \
  FAKE_CURRENT_SNAPSHOT_DIGEST="sha256:$(printf '2%.0s' {1..64})" \
  GITHUB_ACTIONS=true GITHUB_RUN_ID=123456 GITHUB_RUN_ATTEMPT=1 GITHUB_JOB=fixture_quality \
  GITHUB_REPOSITORY=example/fixture GITHUB_SHA="$(printf 'b%.0s' {1..40})" RUNNER_OS=Linux \
  AI_COCKPIT_VERIFICATION_RECEIPT="$case_dir/hosted-runtime-verification.json" \
  AI_COCKPIT_VERIFICATION_ORCHESTRATION="$case_dir/hosted-runtime-orchestration.json" \
  AI_COCKPIT_RUNTIME_BIN="$tmp/runtime-bin" \
  AI_COCKPIT_VERIFICATION_REPOSITORY="$case_dir/repository" \
    python3 - "$case_dir" "$root/tests/ci/run_workspace_package_tests.sh" <<'PY_NATIVE_ENV'
import ast
import hashlib
import json
import os
import re
import stat
import subprocess
import sys
from pathlib import Path
source = Path(sys.argv[2]).read_text()
block = source.split("def coverage_environment_digest():", 1)[1].split("def captured_target_coverage(", 1)[0]
functions = [node for node in ast.parse("def coverage_environment_digest():" + block).body if isinstance(node, ast.FunctionDef)]
namespace = dict(hashlib=hashlib, json=json, os=os, re=re, stat=stat, subprocess=subprocess, Path=Path)
exec(compile(ast.Module(body=functions, type_ignores=[]), "actual-consumer-environment", "exec"), namespace)
p = Path(sys.argv[1]) / "hosted-runtime-orchestration.json"
data = json.loads(p.read_bytes())
if data.get("executionOutput") is not None and "environmentDigest" not in data["executionOutput"]:
    data["executionOutput"]["environmentDigest"] = namespace["coverage_environment_digest"]()
    data["executionOutput"]["cargoContext"] = namespace["cargo_reuse_context"](
        Path(sys.argv[1]) / "repository", str(Path(sys.argv[2]).parent / "run_process_observer_test_runner.sh"))
    p.write_text(json.dumps(data))
PY_NATIVE_ENV
  PACKAGE_LOG="$case_dir/packages.log" WORKSPACE_TEST_WORKERS=1 WORKSPACE_TEST_THREADS=1 \
  FAKE_RUNTIME_LOG="$case_dir/status.log" \
  FAKE_CURRENT_CONTRACT_DIGEST="$(<"$case_dir/contract-digest")" \
  FAKE_CURRENT_SNAPSHOT_DIGEST="sha256:$(printf '2%.0s' {1..64})" \
  GITHUB_ACTIONS=true GITHUB_RUN_ID=123456 GITHUB_RUN_ATTEMPT=1 GITHUB_JOB=fixture_quality \
  GITHUB_REPOSITORY=example/fixture GITHUB_SHA="$(printf 'b%.0s' {1..40})" RUNNER_OS=Linux \
  AI_COCKPIT_VERIFICATION_RECEIPT="$case_dir/hosted-runtime-verification.json" \
  AI_COCKPIT_VERIFICATION_ORCHESTRATION="$case_dir/hosted-runtime-orchestration.json" \
  AI_COCKPIT_RUNTIME_BIN="$tmp/runtime-bin" \
  AI_COCKPIT_VERIFICATION_REPOSITORY="$case_dir/repository" \
    "$root/tests/ci/run_workspace_package_tests.sh" --metadata "$case_dir/metadata.json" \
      --cargo "$tmp/fake-cargo" --report "$case_dir/report.json"
}
make_targeted_native_fixture "$tmp/targeted-native"
run_targeted_native_fixture "$tmp/targeted-native"
jq -e '.state == "passed" and .executedByHostedRuntime == .planned and .executedByCoverageRunner == []' \
  "$tmp/targeted-native/report.json" >/dev/null
test ! -e "$tmp/targeted-native/packages.log"
native_case_count=0
for native_case in all_targets raw_tamper projection_type schema_bool projection_tamper missing_node duplicate_node missing_record duplicate_record \
  command_tamper digest_tamper foreign_wi foreign_runtime foreign_snapshot foreign_run foreign_head \
  filtered unknown_target all_features reused truncated missing_capture warning warning_truncated invalid_hex \
  timeout failure signal cleanup_failure duplicate_target missing_target unsupported_kind symlink wrong_file \
  no_seal local_context environment_changed; do
  case_dir="$tmp/native-$native_case"
  make_targeted_native_fixture "$case_dir"
  python3 - "$case_dir" "$native_case" <<'PY_NATIVE_MUTATE'
import hashlib
import json
import sys
from pathlib import Path
case = Path(sys.argv[1]); mode = sys.argv[2]
formal_path = case / "hosted-runtime-verification.json"
output_path = case / "hosted-runtime-execution.json"
report_path = case / "hosted-runtime-orchestration.json"
contract_path = case / "repository/.ai/work-items/active/WI-HOSTED-RECEIPT.contract.json"
formal = json.loads(formal_path.read_bytes()); output = json.loads(output_path.read_bytes())
report = json.loads(report_path.read_bytes()); contract = json.loads(contract_path.read_bytes())
receipt = formal["receipt"]; record = receipt["executionRecords"][0]
node = output["plannedNodes"][0]
if mode == "all_targets":
    words = contract["verification"][0].split(); words[words.index("--lib")] = "--all-targets"
    contract["verification"][0] = " ".join(words); node["args"] = words[1:]
    p = case / "metadata.json"; metadata = json.loads(p.read_bytes())
    package = next(item for item in metadata["packages"] if item["name"] == "package-a")
    package["targets"].extend([{"name":"extra","kind":["test"],"test":True},
                               {"name":"tool","kind":["bin"],"test":True}])
    p.write_text(json.dumps(metadata))
elif mode == "schema_bool": report["executionOutput"]["schemaVersion"] = True
elif mode == "missing_node": output["plannedNodes"].pop(0)
elif mode == "duplicate_node": output["plannedNodes"].append(node)
elif mode == "missing_record": receipt["executionRecords"].pop(0)
elif mode == "duplicate_record": receipt["executionRecords"].append(record)
elif mode == "command_tamper": node["args"][0] = "clippy"
elif mode == "digest_tamper": node["identityBinding"]["commandDigest"] = "sha256:" + "9" * 64
elif mode.startswith("foreign_"):
    if mode in ("foreign_run", "foreign_head"):
        report["executionOutput"]["providerContext"]["runId" if mode == "foreign_run" else "headSha"] = "9" * (6 if mode == "foreign_run" else 40)
    else:
        key = {"foreign_wi": "workItemId", "foreign_runtime": "runtimeDigest", "foreign_snapshot": "repositorySnapshotDigest"}[mode]
        if key == "workItemId": output["planReceipt"] = dict(output["planReceipt"], workItemId="WI-FOREIGN")
        else: node["identityBinding"][key] = "sha256:" + "9" * 64
elif mode in ("filtered", "unknown_target", "all_features"):
    words = contract["verification"][0].split()
    if mode == "filtered": words.append("only_one_test")
    else:
        i = words.index("--")
        words[i:i] = ["--target", "foreign-unknown-target"] if mode == "unknown_target" else ["--all-features"]
    contract["verification"][0] = " ".join(words)
    node["args"] = words[1:]
elif mode == "reused":
    receipt["results"][0].update(reused=True, action="reuse", satisfiedBy="reused_receipt")
    receipt["executionRecords"].pop(0)
    receipt["planReceipt"].update(executedNodes=["project-command-1"], reusedNodes=["project-command-0"])
    receipt.update(nodesExecuted=1, nodesReused=1)
elif mode == "truncated": record["stdoutTruncated"] = True
elif mode == "missing_capture": del record["stdout"]
elif mode == "warning": record["stderr"] = b"warning: native capture warning\n".hex()
elif mode == "warning_truncated":
    record["stderr"] = b"warning: captured prefix warning\n".hex()
    record["stderrTruncated"] = True
elif mode == "invalid_hex": record["stdout"] = "zz"
elif mode == "timeout": record["timedOut"] = True
elif mode == "failure": record["exitCode"] = 17
elif mode == "signal": record["terminationSignal"] = 9
elif mode == "cleanup_failure": report["failureReason"] = "fixture_cleanup_failed"
elif mode in ("duplicate_target", "missing_target", "unsupported_kind"):
    p = case / "metadata.json"; metadata = json.loads(p.read_bytes())
    package = next(item for item in metadata["packages"] if item["name"] == "package-a")
    if mode == "duplicate_target": package["targets"].append(package["targets"][0])
    elif mode == "missing_target": package["targets"].append({"name":"extra","kind":["test"],"test":True})
    else: package["targets"][0]["kind"] = ["future-unknown-kind"]
    p.write_text(json.dumps(metadata))
elif mode == "wrong_file": report["executionOutput"]["file"] = "../hosted-runtime-execution.json"
elif mode == "local_context": report["executionOutput"]["providerContext"] = None
elif mode == "no_seal": report.pop("executionOutput")
elif mode == "environment_changed": report["executionOutput"]["environmentDigest"] = "sha256:" + "9" * 64
# Reconcile mock native stdout with changed formal records, preserving the
# intended planned-node/projection errors. These are negative test fixtures.
if mode not in ("foreign_wi",):
    for key, value in receipt.items():
        if key != "workItemId": output[key] = value
if mode == "projection_tamper": output["nodesExecuted"] = 0
if mode == "projection_type": output["nodesReused"] = False
contract_path.write_text(json.dumps(contract))
formal["contractDigest"] = "sha256:" + hashlib.sha256(json.dumps(contract, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()).hexdigest()
(case / "contract-digest").write_text(formal["contractDigest"])
formal_raw = json.dumps(formal).encode(); formal_path.write_bytes(formal_raw)
(case / "repository/.ai/evidence/WI-HOSTED-RECEIPT.verification.json").write_bytes(formal_raw)
output_raw = json.dumps(output).encode(); output_path.write_bytes(output_raw)
report["formalReceiptDigest"] = "sha256:" + hashlib.sha256(formal_raw).hexdigest()
if "executionOutput" in report: report["executionOutput"]["sha256"] = "sha256:" + hashlib.sha256(output_raw).hexdigest()
report_path.write_text(json.dumps(report))
if mode == "raw_tamper": output_path.write_bytes(output_raw + b" ")
if mode == "symlink":
    target = case / "moved-output.json"; output_path.rename(target); output_path.symlink_to(target.name)
PY_NATIVE_MUTATE
  native_result=0
  run_targeted_native_fixture "$case_dir" >"$case_dir/check.log" 2>&1 || native_result=$?
  case "$native_case" in
    all_targets)
      test "$native_result" = 0
      test ! -e "$case_dir/packages.log"
      jq -e '.state == "passed" and .executedByHostedRuntime == .planned and (.hostedTargetGroups | length == 4)' "$case_dir/report.json" >/dev/null
      ;;
    all_features|reused|truncated|missing_capture)
      test "$native_result" = 0
      diff -u <(printf '%s\n' package-a) "$case_dir/packages.log"
      jq -e '.state == "passed" and .executedByHostedRuntime == ["package-b"] and .executedByCoverageRunner == ["package-a"]' "$case_dir/report.json" >/dev/null
      ;;
    missing_target)
      test "$native_result" = 0
      diff -u <(printf '%s\n' package-a package-a) "$case_dir/packages.log"
      jq -e '.state == "passed" and .executedByHostedRuntime == ["package-b"] and (.groups | length == 2)' "$case_dir/report.json" >/dev/null
      ;;
    no_seal|local_context|environment_changed)
      test "$native_result" = 0
      diff -u <(printf '%s\n' package-a package-b) "$case_dir/packages.log"
      jq -e '.state == "passed" and .executedByHostedRuntime == [] and .executedByCoverageRunner == .planned' "$case_dir/report.json" >/dev/null
      ;;
    *)
      if [[ "$native_result" == 0 ]]; then
        printf 'targeted native reuse accepted invalid case: %s\n' "$native_case" >&2
        exit 1
      fi
      test ! -e "$case_dir/packages.log"
      jq -e '.state == "failed" and .failurePhase == "hosted_verification_receipt"' "$case_dir/report.json" >/dev/null
      ;;
  esac
  native_case_count=$((native_case_count + 1))
done
printf 'same-job native target reuse cases passed: %s\n' "$native_case_count"

context_case_count=0
for context_case in default_host explicit_host nonhost_env build_target host_build_target parent_config cargo_home relative_cargo_home foreign_runner_config foreign_runner_env; do
  case_dir="$tmp/native-context-$context_case"
  make_targeted_native_fixture "$case_dir"
  mkdir -p "$case_dir/cargo-home"
  context_result=0
  case "$context_case" in
    build_target|host_build_target|foreign_runner_config)
      mkdir -p "$case_dir/repository/.cargo"
      if [[ "$context_case" == foreign_runner_config ]]; then
        printf '[target.%s]\nrunner = "foreign-runner"\n' "$host_target" >"$case_dir/repository/.cargo/config.toml"
      else
        selected_target=aarch64-unknown-linux-gnu
        if [[ "$context_case" == host_build_target ]]; then selected_target=$host_target; fi
        printf '[build]\ntarget = "%s"\n' "$selected_target" >"$case_dir/repository/.cargo/config.toml"
      fi ;;
    parent_config)
      mkdir -p "$case_dir/.cargo"
      printf '[build]\ntarget = "aarch64-unknown-linux-gnu"\n' >"$case_dir/.cargo/config.toml" ;;
    cargo_home)
      printf '[build]\ntarget = "aarch64-unknown-linux-gnu"\n' >"$case_dir/cargo-home/config.toml" ;;
    relative_cargo_home)
      mkdir -p "$case_dir/repository/context-home"
      printf '[build]\ntarget = "aarch64-unknown-linux-gnu"\n' >"$case_dir/repository/context-home/config.toml" ;;
  esac
  case "$context_case" in
    explicit_host) CARGO_BUILD_TARGET="$host_target" run_targeted_native_fixture "$case_dir" >"$case_dir/check.log" 2>&1 || context_result=$? ;;
    nonhost_env) CARGO_BUILD_TARGET=aarch64-unknown-linux-gnu run_targeted_native_fixture "$case_dir" >"$case_dir/check.log" 2>&1 || context_result=$? ;;
    cargo_home) CARGO_HOME="$case_dir/cargo-home" run_targeted_native_fixture "$case_dir" >"$case_dir/check.log" 2>&1 || context_result=$? ;;
    relative_cargo_home) CARGO_HOME=context-home run_targeted_native_fixture "$case_dir" >"$case_dir/check.log" 2>&1 || context_result=$? ;;
    foreign_runner_env) saved_runner=${!runner_variable}; export "$runner_variable=foreign-runner"
      run_targeted_native_fixture "$case_dir" >"$case_dir/check.log" 2>&1 || context_result=$?
      export "$runner_variable=$saved_runner" ;;
    *) run_targeted_native_fixture "$case_dir" >"$case_dir/check.log" 2>&1 || context_result=$? ;;
  esac
  if [[ "$context_case" == foreign_runner_env ]]; then
    test "$context_result" != 0
    jq -e '.state == "failed" and .failurePhase == "runner_setup" and .executed == []' "$case_dir/report.json" >/dev/null
    test ! -e "$case_dir/packages.log"
  elif [[ "$context_case" == default_host || "$context_case" == explicit_host ]]; then
    test "$context_result" = 0
    jq -e '.state == "passed" and .executedByHostedRuntime == .planned and .executedByCoverageRunner == []' "$case_dir/report.json" >/dev/null
    test ! -e "$case_dir/packages.log"
  else
    test "$context_result" = 0
    jq -e '.state == "passed" and .executedByHostedRuntime == [] and .executedByCoverageRunner == .planned' "$case_dir/report.json" >/dev/null
    diff -u <(printf '%s\n' package-a package-b) "$case_dir/packages.log"
  fi
  context_case_count=$((context_case_count + 1))
done
printf 'Cargo target/config/runner context cases passed: %s\n' "$context_case_count"


cp "$tmp/hosted-verification.json" "$tmp/valid-targeted-verification.json"
cp "$tmp/hosted-repository/.ai/work-items/active/WI-HOSTED-RECEIPT.contract.json" "$tmp/valid-targeted-contract.json"
cat >"$tmp/slow-cargo" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$3" >>"$PACKAGE_LOG"
printf '%s\n' "$$" >"$PACKAGE_LOG.pid"
exec sleep 10
SH
cat >"$tmp/warning-cargo" <<'SH'
#!/usr/bin/env bash
printf '%s\n' "$3" >>"$PACKAGE_LOG"
printf 'warning: a genuine fresh package diagnostic\n' >&2
SH
chmod +x "$tmp/slow-cargo" "$tmp/warning-cargo"
targeted_fallback_case_count=0
for fallback_case in full_package multi_targets null_manifest reused_targeted \
  workspace_without_manifest filtered wrapped env_options nested_env filtered_plus_valid workspace_plus_valid \
  foreign_package bad_digest failed_result \
  missing_result duplicate_result bad_accounting wrong_contract wrong_snapshot wrong_runtime \
  mismatched_original malformed_manifest native_timed_out no_tests missing_record \
  native_reuse_inconsistent runtime_stale fresh_failure fresh_timeout fresh_warning; do
  cp "$tmp/valid-targeted-verification.json" "$tmp/hosted-verification.json"
  cp "$tmp/valid-targeted-contract.json" "$tmp/hosted-repository/.ai/work-items/active/WI-HOSTED-RECEIPT.contract.json"
  python3 - "$fallback_case" "$tmp/hosted-verification.json" \
    "$tmp/hosted-repository/.ai/evidence/WI-HOSTED-RECEIPT.verification.json" \
    "$tmp/hosted-repository/.ai/work-items/active/WI-HOSTED-RECEIPT.contract.json" \
    "$tmp/contract-digest" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

case = sys.argv[1]
receipt_path, evidence_path, contract_path, digest_path = map(Path, sys.argv[2:])
formal = json.loads(receipt_path.read_text())
receipt = formal["receipt"]
plan = receipt["planReceipt"]
contract = json.loads(contract_path.read_text())
node = "project-command-2"
record = next(r for r in receipt["executionRecords"] if r["nodeId"] == node)
result = next(r for r in receipt["results"] if r["nodeId"] == node)
commands = {
    "full_package": "cargo test --locked -p package-a --all-targets -- --test-threads=1",
    "multi_targets": "cargo test --locked --package package-b --lib --bin app --test rpc --test unit -- --test-threads 1",
    "workspace_without_manifest": "cargo test --locked --workspace -- --test-threads=1",
    "filtered": "cargo test --locked -p package-a --lib -- one_test",
    "wrapped": "FOO=1 cargo test --locked -p package-a --lib",
    "env_options": "env -i cargo test --locked -p package-a --lib",
    "nested_env": "env FOO=1 env BAR=2 cargo test --locked -p package-a --lib",
    "foreign_package": "cargo test --locked -p foreign --all-targets",
    "no_tests": "cargo fmt --all -- --check",
}
if case in commands:
    contract["verification"][2] = commands[case]
if case in {"filtered_plus_valid", "workspace_plus_valid"}:
    contract["verification"].append("cargo test --locked -p package-a -- one_test" if case == "filtered_plus_valid" else "cargo test --locked --workspace")
if case == "null_manifest":
    plan["coverageManifest"] = None
elif case == "malformed_manifest":
    plan["coverageManifest"] = {}
elif case == "bad_digest":
    record["commandDigest"] = "not-a-digest"
elif case == "native_timed_out":
    record["timedOut"] = True
elif case == "failed_result":
    result["passed"] = False
elif case == "missing_result":
    receipt["results"].remove(result)
    receipt["executionRecords"].remove(record)
    plan["executedNodes"].remove(node)
    receipt["nodesPlanned"] -= 1
    receipt["nodesExecuted"] -= 1
elif case == "duplicate_result":
    receipt["results"].append(dict(result))
    receipt["nodesPlanned"] += 1
    receipt["nodesExecuted"] += 1
elif case == "bad_accounting":
    receipt["nodesExecuted"] -= 1
elif case == "missing_record":
    receipt["executionRecords"].remove(record)
elif case in {"reused_targeted", "native_reuse_inconsistent"}:
    plan["executedNodes"].remove(node)
    plan["reusedNodes"] = [node]
    receipt["nodesExecuted"] -= 1
    receipt["nodesReused"] += 1
    if case == "reused_targeted":
        result.update(reused=True, action="reuse", satisfiedBy="reused_receipt")
        receipt["executionRecords"].remove(record)
contract_path.write_text(json.dumps(contract))
digest = "sha256:" + hashlib.sha256(json.dumps(contract, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()).hexdigest()
digest_path.write_text(digest)
formal["contractDigest"] = digest
if case == "wrong_contract":
    formal["contractDigest"] = "sha256:" + "f" * 64
elif case == "wrong_snapshot":
    formal["repositorySnapshotDigest"] = plan["repositorySnapshotDigest"] = "sha256:" + "f" * 64
elif case == "wrong_runtime":
    formal["runtimeDigest"] = "sha256:" + "f" * 64
raw = json.dumps(formal).encode()
receipt_path.write_bytes(raw)
evidence_path.write_bytes(raw)
if case == "mismatched_original":
    evidence_path.write_bytes(raw + b"\n")
PY
  refresh_hosted_orchestration passed
  fallback_cargo="$tmp/fake-cargo"
  freshness=fresh
  fallback_timeout=900
  case "$fallback_case" in
    runtime_stale) freshness=stale ;;
    fresh_failure) fallback_cargo="$tmp/failing-diagnostic-cargo" ;;
    fresh_timeout) fallback_cargo="$tmp/slow-cargo"; fallback_timeout=1 ;;
    fresh_warning) fallback_cargo="$tmp/warning-cargo" ;;
  esac
  fallback_exit=0
  PACKAGE_LOG="$tmp/fallback-$fallback_case-packages.log" WORKSPACE_TEST_WORKERS=1 WORKSPACE_TEST_THREADS=1 \
  WORKSPACE_TEST_TIMEOUT_SECONDS="$fallback_timeout" FAKE_FRESHNESS_STATE="$freshness" \
  FAKE_RUNTIME_LOG="$tmp/runtime-status.log" \
  FAKE_CURRENT_CONTRACT_DIGEST="$(<"$tmp/contract-digest")" \
  FAKE_CURRENT_SNAPSHOT_DIGEST="sha256:$(printf '2%.0s' {1..64})" \
  AI_COCKPIT_VERIFICATION_RECEIPT="$tmp/hosted-verification.json" \
  AI_COCKPIT_RUNTIME_BIN="$tmp/runtime-bin" \
  AI_COCKPIT_VERIFICATION_REPOSITORY="$tmp/hosted-repository" \
    "$root/tests/ci/run_workspace_package_tests.sh" --metadata "$tmp/metadata.json" \
      --cargo "$fallback_cargo" --report "$tmp/fallback-$fallback_case-report.json" \
      >"$tmp/fallback-$fallback_case.stdout" 2>"$tmp/fallback-$fallback_case.stderr" || fallback_exit=$?
  case "$fallback_case" in
    full_package|multi_targets|null_manifest|reused_targeted)
      test "$fallback_exit" == 0 || { cat "$tmp/fallback-$fallback_case.stderr" >&2; exit 1; }
      diff -u <(printf '%s\n' package-a package-b) "$tmp/fallback-$fallback_case-packages.log"
      jq -e '.state == "passed" and .executedByHostedRuntime == [] and .executedByCoverageRunner == .planned' \
        "$tmp/fallback-$fallback_case-report.json" >/dev/null
      ;;
    fresh_failure|fresh_timeout|fresh_warning)
      test "$fallback_exit" != 0
      jq -e '.state == "failed" and .failurePhase == "package_test" and .failedPackage == "package-a" and .executed == [] and .omitted == .planned' \
        "$tmp/fallback-$fallback_case-report.json" >/dev/null
      if [[ "$fallback_case" == fresh_timeout ]]; then
        jq -e '.failedExitCode == 124 and .groupTimeoutSeconds == 1' "$tmp/fallback-$fallback_case-report.json" >/dev/null
        diff -u <(printf '%s\n' package-a) "$tmp/fallback-$fallback_case-packages.log"
        test -f "$tmp/fallback-$fallback_case-packages.log.pid"
        ! kill -0 "$(<"$tmp/fallback-$fallback_case-packages.log.pid")" 2>/dev/null
      fi
      ;;
    *)
      test "$fallback_exit" != 0
      jq -e '.state == "failed" and .failurePhase == "hosted_verification_receipt" and .executed == []' \
        "$tmp/fallback-$fallback_case-report.json" >/dev/null
      test ! -e "$tmp/fallback-$fallback_case-packages.log"
      ;;
  esac
  targeted_fallback_case_count=$((targeted_fallback_case_count + 1))
done
printf 'targeted fallback regression cases passed: %s\n' "$targeted_fallback_case_count"
for invalid_timeout in 0 901 1.5 invalid; do
  if WORKSPACE_TEST_TIMEOUT_SECONDS="$invalid_timeout" "$root/tests/ci/run_workspace_package_tests.sh" \
      --metadata "$tmp/metadata.json" --cargo "$tmp/fake-cargo" --report "$tmp/invalid-timeout-report.json" \
      >"$tmp/invalid-timeout.stdout" 2>"$tmp/invalid-timeout.stderr"; then
    printf 'coverage accepted an invalid timeout: %s\n' "$invalid_timeout" >&2
    exit 1
  fi
  grep -q 'timeout must be an integer from 1 to 900' "$tmp/invalid-timeout.stderr"
done

# The grouping source is metadata, never the hosted command's package names.
mkdir -p "$tmp/grouped-package"
cat >"$tmp/grouped-package/Cargo.toml" <<'TOML'
[package]
name = "package-a"
version = "1.0.0"
[[test]]
name = "integration"
test = false
bench = true
[[test]]
name = "disabled"
test = false
bench = false
[[bench]]
name = "performance"
test = false
TOML
python3 - "$tmp/metadata.json" "$tmp/grouped-metadata.json" "$tmp/grouped-package/Cargo.toml" <<'PY'
import json
import sys
from pathlib import Path

metadata = json.loads(Path(sys.argv[1]).read_text())
package = next(p for p in metadata["packages"] if p["name"] == "package-a")
package["manifest_path"] = sys.argv[3]
package["targets"] = [
    {"name": "library", "kind": ["lib"], "test": False},
    {"name": "app", "kind": ["bin"], "test": False},
    {"name": "integration", "kind": ["test"], "test": False},
    {"name": "sample", "kind": ["example"], "test": False},
    {"name": "performance", "kind": ["bench"], "test": False},
    {"name": "disabled", "kind": ["test"], "test": False},
    {"name": "build-script-build", "kind": ["custom-build"], "test": False},
]
Path(sys.argv[2]).write_text(json.dumps(metadata))
PY
cat >"$tmp/group-cargo" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
python3 - "$GROUP_LOG" "$@" <<'PY'
import json
import sys
with open(sys.argv[1], "a", encoding="utf-8") as stream:
    stream.write(json.dumps(sys.argv[2:]) + "\n")
PY
SH
chmod +x "$tmp/group-cargo"
GROUP_LOG="$tmp/groups.log" AI_COCKPIT_VERIFICATION_RECEIPT= \
WORKSPACE_TEST_WORKERS=1 WORKSPACE_TEST_THREADS=1 \
  "$root/tests/ci/run_workspace_package_tests.sh" --metadata "$tmp/grouped-metadata.json" \
    --cargo "$tmp/group-cargo" --report "$tmp/grouped-report.json"
python3 - "$tmp/groups.log" "$tmp/grouped-report.json" <<'PY'
import json
import sys
from pathlib import Path

actual = [json.loads(line) for line in Path(sys.argv[1]).read_text().splitlines()]
assert len(actual) == 6, actual
assert [args[5:args.index("--")] for args in actual] == [
    ["--lib"], ["--bin", "app"], ["--test", "integration"],
    ["--example", "sample"], ["--bench", "performance"], ["--lib"],
], actual
report = json.loads(Path(sys.argv[2]).read_text())
assert report["state"] == "passed" and report["executed"] == ["package-a", "package-b"]
assert len(report["groups"]) == 6 and all(g["state"] == "passed" for g in report["groups"])
assert sum(not t["selected"] for t in report["targetInventory"]) == 2
PY

# No malformed / incomplete inventory may become empty or partial green.
inventory_case_count=0
for inventory_case in missing_targets empty_targets duplicate_target unknown_kind bad_test_flag required_feature disabled_only inconsistent_manifest; do
  python3 - "$tmp/grouped-metadata.json" "$tmp/invalid-inventory.json" "$inventory_case" <<'PY'
import json
import sys
from pathlib import Path
m = json.loads(Path(sys.argv[1]).read_text())
p = next(p for p in m["packages"] if p["name"] == "package-a")
case = sys.argv[3]
if case == "missing_targets": del p["targets"]
elif case == "empty_targets": p["targets"] = []
elif case == "duplicate_target": p["targets"].append(dict(p["targets"][0]))
elif case == "unknown_kind": p["targets"][0]["kind"] = ["future-kind"]
elif case == "bad_test_flag": p["targets"][0]["test"] = "false"
elif case == "required_feature": p["targets"][0]["required-features"] = ["not-proven-enabled"]
elif case == "disabled_only": p["targets"] = [t for t in p["targets"] if t["name"] == "disabled"]
elif case == "inconsistent_manifest": p["targets"].append({"name":"missing-declaration","kind":["test"],"test":False})
Path(sys.argv[2]).write_text(json.dumps(m))
PY
  if GROUP_LOG="$tmp/invalid-$inventory_case-groups.log" AI_COCKPIT_VERIFICATION_RECEIPT= \
    "$root/tests/ci/run_workspace_package_tests.sh" --metadata "$tmp/invalid-inventory.json" \
      --cargo "$tmp/group-cargo" --report "$tmp/invalid-$inventory_case-report.json" \
      >"$tmp/invalid-$inventory_case.stdout" 2>"$tmp/invalid-$inventory_case.stderr"; then
    printf 'coverage accepted invalid target inventory: %s\n' "$inventory_case" >&2
    exit 1
  fi
  jq -e '.state == "failed" and .failurePhase == "target_inventory" and .executed == []' \
    "$tmp/invalid-$inventory_case-report.json" >/dev/null
  test ! -e "$tmp/invalid-$inventory_case-groups.log"
  inventory_case_count=$((inventory_case_count + 1))
done
printf 'target inventory rejection cases passed: %s\n' "$inventory_case_count"

# A failing first worker waits for the already-running sibling and never
# launches another group. This exercises actual concurrent fake processes.
cat >"$tmp/two-worker-cargo" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$3" >>"$GROUP_EVENT_LOG.started"
if [[ "$3" == package-a ]]; then
  for ((i=0; i<500; i++)); do
    [[ -f "$GROUP_EVENT_LOG.running" ]] && exit 17
    sleep 0.01
  done
  exit 19
fi
printf '%s\n' "$$" >"$GROUP_EVENT_LOG.running"
sleep 0.2
printf '%s\n' finished >"$GROUP_EVENT_LOG.finished"
SH
chmod +x "$tmp/two-worker-cargo"
python3 - "$tmp/metadata.json" "$tmp/two-worker-metadata.json" <<'PY'
import json
import sys
from pathlib import Path
m = json.loads(Path(sys.argv[1]).read_text())
m["packages"].append({"name":"package-c","source":None,"targets":[{"name":"package_c","kind":["lib"],"test":True}]})
Path(sys.argv[2]).write_text(json.dumps(m))
PY
if GROUP_EVENT_LOG="$tmp/two-worker" AI_COCKPIT_VERIFICATION_RECEIPT= \
  WORKSPACE_TEST_WORKERS=2 WORKSPACE_TEST_THREADS=1 \
    "$root/tests/ci/run_workspace_package_tests.sh" --metadata "$tmp/two-worker-metadata.json" \
      --cargo "$tmp/two-worker-cargo" --report "$tmp/two-worker-report.json" \
      >"$tmp/two-worker.stdout" 2>"$tmp/two-worker.stderr"; then
  printf 'coverage accepted a concurrent failed group\n' >&2
  exit 1
fi
test -f "$tmp/two-worker.finished"
! kill -0 "$(<"$tmp/two-worker.running")" 2>/dev/null
jq -e '.state == "failed" and .failedPackage == "package-a" and .executed == ["package-b"] and .omitted == ["package-a", "package-c"] and (.groups | map(select(.state == "not_run")) | length) == 1' \
  "$tmp/two-worker-report.json" >/dev/null
diff -u <(printf '%s\n' package-a package-b) <(sort "$tmp/two-worker.started")

# Ignore TERM in the command process itself; GNU timeout must reach its finite
# KILL boundary. This is not an arbitrary detached-descendant cleanup claim.
cat >"$tmp/ignore-term-cargo" <<'SH'
#!/usr/bin/env bash
exec python3 - "$GROUP_PID_FILE" <<'PY'
import os
import signal
import sys
from pathlib import Path
signal.signal(signal.SIGTERM, signal.SIG_IGN)
Path(sys.argv[1]).write_text(str(os.getpid()))
while True:
    signal.pause()
PY
SH
chmod +x "$tmp/ignore-term-cargo"
if GROUP_PID_FILE="$tmp/ignore-term.pid" AI_COCKPIT_VERIFICATION_RECEIPT= \
  WORKSPACE_TEST_WORKERS=1 WORKSPACE_TEST_TIMEOUT_SECONDS=1 \
    "$root/tests/ci/run_workspace_package_tests.sh" --metadata "$tmp/metadata.json" \
      --cargo "$tmp/ignore-term-cargo" --report "$tmp/ignore-term-report.json" \
      >"$tmp/ignore-term.stdout" 2>"$tmp/ignore-term.stderr"; then
  printf 'coverage accepted a KILL-terminated group\n' >&2
  exit 1
fi
jq -e '.state == "failed" and .failedExitCode == 137 and .executed == [] and (.groups | map(select(.state == "not_run")) | length) == 1' \
  "$tmp/ignore-term-report.json" >/dev/null
python3 - "$tmp/ignore-term.pid" <<'PY'
import sys
from pathlib import Path
pid = Path(sys.argv[1]).read_text()
stat = Path("/proc") / pid / "stat"
assert not stat.exists(), "KILL fixture process still present: " + (stat.read_text() if stat.exists() else pid)
PY
printf 'group concurrency and TERM/KILL regressions passed\n'

# Exercise the actual owner reaper with an in-memory OS/clock. In particular,
# an endless stream of exited children must not bypass the cleanup deadline.
python3 - "$root/tests/ci/run_workspace_package_tests.sh" <<'PY'
import ast
import errno
import io
import types
import sys
from pathlib import Path

source = Path(sys.argv[1]).read_text().split('cat >"$tmp/group-owner.py" <<\'PY\'\n', 1)[1].split('\nPY\n', 1)[0]
tree = ast.parse(source)
function = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == "reap_group_children")
code = compile(ast.fix_missing_locations(ast.Module(body=[function], type_ignores=[])), "actual-group-owner", "exec")
for case in ["echild", "reaped_then_echild", "alive", "continuous_exits", "wait_error"]:
    clock = [0]
    calls = [0]
    def monotonic():
        clock[0] += 1
        return clock[0]
    def waitpid(pid, flags):
        assert pid == -1
        calls[0] += 1
        assert calls[0] <= 5, "reaping failed to respect its deadline"
        if case == "wait_error": raise OSError(errno.EIO, "injected")
        if case == "echild" or (case == "reaped_then_echild" and calls[0] > 1): raise ChildProcessError()
        return (123, 0) if case in {"continuous_exits", "reaped_then_echild"} else (0, 0)
    scope = dict(os=types.SimpleNamespace(waitpid=waitpid, WNOHANG=1),
                 time=types.SimpleNamespace(monotonic=monotonic, sleep=lambda _: None),
                 sys=types.SimpleNamespace(stderr=io.StringIO()), errno=errno)
    exec(code, scope)
    failure = None
    try: scope["reap_group_children"](4)
    except SystemExit as error: failure = str(error)
    if case in {"echild", "reaped_then_echild"}: assert failure is None
    else: assert failure and failure.startswith("target-group cleanup uncertain:")
watchdog_guard = next(n for n in tree.body if isinstance(n, ast.If) and isinstance(n.test, ast.Name) and n.test.id == "watchdog_failed")
for failed in [False, True]:
    failure = None
    try:
        exec(compile(ast.fix_missing_locations(ast.Module(body=[watchdog_guard], type_ignores=[])), "actual-owner-watchdog-guard", "exec"), {"watchdog_failed": failed})
    except SystemExit as error: failure = str(error)
    assert bool(failure) == failed
print("owner bounded-reaping and watchdog logic passed: 7 cases")
guard = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == "kill_held_group")
guard_code = compile(ast.fix_missing_locations(ast.Module(body=[guard], type_ignores=[])), "actual-owner-group-guard", "exec")
for case in ["owned", "reaped", "not_child", "foreign_pgid", "foreign_sid", "signal_error"]:
    masks = []
    kills = []
    def mask(how, values):
        masks.append(how)
        return {"previous"}
    def waitid(kind, pid, flags):
        assert (kind, pid, flags) == (1, 123, 14)
        if case == "not_child": raise ChildProcessError()
    def killpg(pid, signum):
        assert (pid, signum) == (123, 9)
        if case == "signal_error": raise PermissionError()
        kills.append((pid, signum))
    scope = dict(signal=types.SimpleNamespace(pthread_sigmask=mask, SIG_BLOCK=0, SIG_SETMASK=2, SIGTERM=15, SIGINT=2, SIGKILL=9),
                 os=types.SimpleNamespace(getpgrp=lambda: 999, waitid=waitid, P_PID=1, WEXITED=2, WNOHANG=4, WNOWAIT=8,
                                          getpgid=lambda _: 777 if case == "foreign_pgid" else 123,
                                          getsid=lambda _: 888 if case == "foreign_sid" else 123, killpg=killpg))
    exec(guard_code, scope)
    result = scope["kill_held_group"](types.SimpleNamespace(pid=123, returncode=0 if case == "reaped" else None))
    assert result is (case == "owned") and kills == ([(123, 9)] if case == "owned" else [])
    assert masks == [0, 2], "the signal mask must always be restored"
print("held command-group ownership guard passed: 6 cases")
PY

# Signal the live, identity-checked private owner, not a guessed process group.
# A missing forwarding path must fail after the bounded command finishes.
cat >"$tmp/signal-cargo" <<'SH'
#!/usr/bin/env bash
exec python3 - "$GROUP_SIGNAL_MARKER" <<'PY'
import json
import os
import time
import sys
from pathlib import Path
timer = os.getppid()
stat = Path(f"/proc/{timer}/stat").read_text()
owner = int(stat[stat.rfind(")") + 2:].split()[1])
raw = Path(f"/proc/{owner}/stat").read_text()
Path(sys.argv[1]).write_text(json.dumps({"owner": owner, "starttime": raw[raw.rfind(")") + 2:].split()[19], "command": os.getpid()}))
time.sleep(30)
PY
SH
chmod +x "$tmp/signal-cargo"
python3 - "$root/tests/ci/run_workspace_package_tests.sh" "$tmp/metadata.json" "$tmp/signal-cargo" "$tmp" <<'PY'
import json
import os
import signal
import subprocess
import sys
import time
from pathlib import Path

runner, metadata, cargo, temporary = sys.argv[1:]
for signum in [signal.SIGTERM, signal.SIGINT]:
    prefix = Path(temporary) / f"owner-signal-{signum}"
    marker = Path(str(prefix) + ".marker")
    report = Path(str(prefix) + ".json")
    env = dict(os.environ, AI_COCKPIT_VERIFICATION_RECEIPT="", WORKSPACE_TEST_WORKERS="1",
               WORKSPACE_TEST_TIMEOUT_SECONDS="20", GROUP_SIGNAL_MARKER=str(marker))
    with Path(str(prefix) + ".log").open("wb") as stream:
        process = subprocess.Popen([runner, "--metadata", metadata, "--cargo", cargo, "--report", str(report)], env=env, stdout=stream, stderr=subprocess.STDOUT)
        deadline = time.monotonic() + 10
        while not marker.is_file() and time.monotonic() < deadline and process.poll() is None:
            time.sleep(0.01)
        assert marker.is_file(), "target group did not reach the signal fixture"
        entry = json.loads(marker.read_text())
        raw = Path(f"/proc/{entry['owner']}/stat").read_text()
        assert raw[raw.rfind(")") + 2:].split()[19] == entry["starttime"]
        started = time.monotonic()
        os.kill(entry["owner"], signum)
        code = process.wait(timeout=60)
        assert code != 0 and time.monotonic() - started < 10, "owner did not promptly forward cancellation"
    observed = json.loads(report.read_text())
    assert observed["state"] == "failed" and observed["failedExitCode"] == 128 + signum
    assert observed["executed"] == [] and len([g for g in observed["groups"] if g["state"] == "not_run"]) == 1
    assert not Path(f"/proc/{entry['owner']}").exists()
    assert not Path(f"/proc/{entry['command']}").exists()
print("owner TERM/INT forwarding passed: 2 cases")
PY

# A timer that never enforces its deadline must not leave the actual command
# and its same-group descendant alive when the owner reports failure.
mkdir -p "$tmp/broken-watchdog-tools"
cat >"$tmp/broken-watchdog-tools/timeout" <<'PY'
#!/usr/bin/env python3
import subprocess
import sys
assert sys.argv[1:3] == ["--signal=TERM", "--kill-after=30s"]
assert sys.argv[3] == "1s"
child = subprocess.Popen(sys.argv[4:])
raise SystemExit(child.wait())
PY
cat >"$tmp/watchdog-command" <<'PY'
#!/usr/bin/env python3
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time
signal.signal(signal.SIGTERM, signal.SIG_IGN)
child = subprocess.Popen([sys.executable, "-c", "import signal,time; signal.signal(signal.SIGTERM, signal.SIG_IGN); time.sleep(90)"])
identities = []
for pid in [os.getpid(), child.pid]:
    stat = Path(f"/proc/{pid}/stat").read_text()
    fields = stat[stat.rfind(")") + 2:].split()
    identities.append({"pid": pid, "starttime": fields[19]})
Path(os.environ["BROKEN_WATCHDOG_MARKER"]).write_text(json.dumps(identities))
time.sleep(90)
PY
chmod +x "$tmp/broken-watchdog-tools/timeout" "$tmp/watchdog-command"
watchdog_exit=0
PATH="$tmp/broken-watchdog-tools:$PATH" BROKEN_WATCHDOG_MARKER="$tmp/watchdog-command-pids.json" \
AI_COCKPIT_VERIFICATION_RECEIPT= WORKSPACE_TEST_WORKERS=1 WORKSPACE_TEST_TIMEOUT_SECONDS=1 \
  "$root/tests/ci/run_workspace_package_tests.sh" --metadata "$tmp/metadata.json" \
    --cargo "$tmp/watchdog-command" --report "$tmp/broken-watchdog-report.json" \
    >"$tmp/broken-watchdog.stdout" 2>"$tmp/broken-watchdog.stderr" || watchdog_exit=$?
test "$watchdog_exit" != 0
python3 - "$tmp/broken-watchdog-report.json" "$tmp/watchdog-command-pids.json" <<'PY'
import json
import sys
from pathlib import Path
report = json.loads(Path(sys.argv[1]).read_text())
assert report["state"] == "failed" and report["executed"] == []
assert "target-group cleanup uncertain:" in report["failureDiagnosticTail"]
assert len([g for g in report["groups"] if g["state"] == "not_run"]) == 1
remaining = []
for identity in json.loads(Path(sys.argv[2]).read_text()):
    path = Path(f"/proc/{identity['pid']}/stat")
    if path.exists():
        raw = path.read_text()
        fields = raw[raw.rfind(")") + 2:].split()
        if fields[19] == identity["starttime"]:
            remaining.append(dict(identity, state=fields[0], ppid=fields[1]))
print(json.dumps({"brokenWatchdogState": report["state"], "remainingOriginalProcesses": remaining}))
assert not remaining, "broken watchdog left the owned command tree unreaped"
PY
printf 'real broken-watchdog command-group cleanup regression passed\n'

printf 'workspace package coverage regression passed\n'
