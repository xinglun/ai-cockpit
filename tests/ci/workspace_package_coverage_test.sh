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

cat >"$tmp/metadata.json" <<'JSON'
{"packages":[
  {"name":"package-b","source":null,"version":"1.0.0"},
  {"name":"external","source":"registry+example","version":"1.0.0"},
  {"name":"package-a","source":null,"version":"1.0.0"}
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
mkdir -p "$tmp/hosted-repository/.ai/evidence"
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
    "print(json.dumps({'workItemId': 'WI-HOSTED-RECEIPT', 'verification': 'verified', 'evidenceFreshness': {'state': 'fresh'}, 'sourceDigests': {'contract': os.environ['FAKE_CURRENT_CONTRACT_DIGEST'], 'repositorySnapshot': os.environ['FAKE_CURRENT_SNAPSHOT_DIGEST']}}))\n",
    encoding="utf-8",
)
runtime_path.chmod(0o755)
metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
packages = sorted({p["name"] for p in metadata["packages"] if p.get("source") is None})
runtime_digest = "sha256:" + hashlib.sha256(runtime_path.read_bytes()).hexdigest()
repository_id = "sha256:" + "1" * 64
snapshot_digest = "sha256:" + "2" * 64
work_item_id = "WI-HOSTED-RECEIPT"
receipt = {
    "workItemId": work_item_id,
    "passed": True,
    "runtimeDigest": runtime_digest,
    "runtimeVersion": "0.2.113",
    "repositoryId": repository_id,
    "nodesPlanned": len(packages) + 1,
    "nodesExecuted": len(packages) + 1,
    "nodesReused": 0,
    "results": [
        {"nodeId": f"project-command-0-package-{package}", "passed": True}
        for package in packages
    ] + [{"nodeId": "project-command-1", "passed": True}],
    "planReceipt": {
        "workItemId": work_item_id,
        "repositoryId": repository_id,
        "repositorySnapshotDigest": snapshot_digest,
        "executedNodes": [f"project-command-0-package-{package}" for package in packages]
        + ["project-command-1"],
        "reusedNodes": [],
        "coverageManifest": {
            "workspaceMembers": packages,
            "nodeIds": [f"project-command-0-package-{package}" for package in packages],
        },
    },
}
evidence = {
    "workItemId": work_item_id,
    "passed": True,
    "runtimeDigest": runtime_digest,
    "runtimeVersion": "0.2.113",
    "contractDigest": "sha256:" + "3" * 64,
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
    "$tmp/hosted-orchestration.json" "$verification_state" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

runtime = Path(sys.argv[1])
receipt = Path(sys.argv[2])
repository = Path(sys.argv[3])
orchestration = Path(sys.argv[4])
state = sys.argv[5]
formal_bytes = receipt.read_bytes()
formal = json.loads(formal_bytes)
orchestration.write_text(
    json.dumps({
        "schemaVersion": 1,
        "workItemId": formal["workItemId"],
        "runtimeDigest": "sha256:" + hashlib.sha256(runtime.read_bytes()).hexdigest(),
        "formalReceiptDigest": "sha256:" + hashlib.sha256(formal_bytes).hexdigest(),
        "executionRepository": str(repository.resolve()),
        "verificationState": state,
    }),
    encoding="utf-8",
)
PY
}
refresh_hosted_orchestration passed
export AI_COCKPIT_VERIFICATION_ORCHESTRATION="$tmp/hosted-orchestration.json"
PACKAGE_LOG="$tmp/hosted-packages.log" \
FAKE_RUNTIME_LOG="$tmp/runtime-status.log" \
FAKE_CURRENT_CONTRACT_DIGEST="sha256:$(printf '3%.0s' {1..64})" \
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

# When Runtime reused only one package node, the coverage consumer runs just
# that node and retains the current hosted execution for the other package.
python3 - "$tmp/hosted-verification.json" "$tmp/hosted-repository/.ai/evidence/WI-HOSTED-RECEIPT.verification.json" <<'PY'
import json
import sys
from pathlib import Path

receipt_path, evidence_path = map(Path, sys.argv[1:])
formal = json.loads(receipt_path.read_text(encoding="utf-8"))
receipt = formal["receipt"]
reused_id = "project-command-0-package-package-b"
receipt["nodesReused"] = 1
receipt["nodesExecuted"] = len(receipt["results"]) - 1
receipt["planReceipt"]["reusedNodes"] = [reused_id]
receipt["planReceipt"]["executedNodes"] = [
    result["nodeId"] for result in receipt["results"] if result["nodeId"] != reused_id
]
for result in receipt["results"]:
    result["reused"] = result["nodeId"] == reused_id
encoded = json.dumps(formal).encode("utf-8")
receipt_path.write_bytes(encoded)
evidence_path.write_bytes(encoded)
PY
refresh_hosted_orchestration passed
PACKAGE_LOG="$tmp/partial-hosted-packages.log" \
FAKE_RUNTIME_LOG="$tmp/runtime-status.log" \
FAKE_CURRENT_CONTRACT_DIGEST="sha256:$(printf '3%.0s' {1..64})" \
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
FAKE_CURRENT_CONTRACT_DIGEST="sha256:$(printf '3%.0s' {1..64})" \
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
  FAKE_CURRENT_CONTRACT_DIGEST="sha256:$(printf '3%.0s' {1..64})" \
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
  FAKE_CURRENT_CONTRACT_DIGEST="sha256:$(printf '3%.0s' {1..64})" \
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

printf 'workspace package coverage regression passed\n'
