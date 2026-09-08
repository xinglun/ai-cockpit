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
python3 - "$gate" "$fixtures/p0-baseline.json" "$fixtures/p0-candidate-pass.json" <<'PY'
import json
import pathlib
import subprocess
import sys
import tempfile

gate, baseline, candidate = sys.argv[1:]
with tempfile.TemporaryDirectory() as directory:
    forged_path = pathlib.Path(directory) / "forged.json"
    forged = json.loads(pathlib.Path(candidate).read_text(encoding="utf-8"))
    forged["environment"]["repository"]["dirty"] = True
    forged_path.write_text(json.dumps(forged), encoding="utf-8")
    result = subprocess.run([gate, baseline, str(forged_path)], capture_output=True, text=True)
    if result.returncode == 0:
        raise SystemExit("P0 gate accepted a forged clean scenario")
print("P0 gate rejected a forged scenario")
PY
echo "P0 performance regression gate passed identity, evidence, and negative checks"
