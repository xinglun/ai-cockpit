#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
gate="$root/tests/performance/p0_regression_gate.sh"
fixtures="$root/tests/performance/fixtures"

"$gate" "$fixtures/p0-baseline.json" "$fixtures/p0-candidate-pass.json" >/dev/null
if "$gate" "$fixtures/p0-baseline.json" "$fixtures/p0-candidate-budget-fail.json" >/dev/null 2>&1; then
  echo "P0 gate accepted a budget regression" >&2
  exit 1
fi
if "$gate" "$fixtures/p0-baseline.json" "$fixtures/p0-candidate-insufficient-percentile.json" >/dev/null 2>&1; then
  echo "P0 gate accepted an unreliable percentile" >&2
  exit 1
fi
if "$gate" "$fixtures/p0-baseline.json" "$fixtures/p0-candidate-incomparable-environment.json" >/dev/null 2>&1; then
  echo "P0 gate accepted incomparable environments" >&2
  exit 1
fi
echo "P0 performance regression gate passed identity, evidence, and negative checks"
