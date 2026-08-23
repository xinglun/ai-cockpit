#!/usr/bin/env bash
set -euo pipefail

repo=${1:-$(pwd)}
report=${2:-$repo/target/governance-integrity-report.json}
python3 "$repo/tests/ci/governance_integrity_gate.py" --repo "$repo" --report "$report"
