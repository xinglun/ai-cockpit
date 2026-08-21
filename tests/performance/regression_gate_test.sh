#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
gate="$root/tests/performance/regression_gate.sh"
fixtures="$root/tests/performance/fixtures"

"$gate" "$fixtures/baseline.json" "$fixtures/candidate-pass.json" >/dev/null
if "$gate" "$fixtures/baseline.json" "$fixtures/candidate-fail.json" >/dev/null 2>&1; then
  echo "regression gate accepted an identity/budget failure" >&2
  exit 1
fi
echo "performance regression gate passed negative and positive checks"
