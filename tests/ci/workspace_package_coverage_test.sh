#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/workspace-package-coverage.XXXXXX")
trap 'rm -rf "$tmp"' EXIT

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

printf 'workspace package coverage regression passed\n'
