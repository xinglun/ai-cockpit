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
python3 - "$gate" "$fixtures/p0-baseline.json" <<'PY'
import json
import pathlib
import subprocess
import sys
import tempfile

gate, baseline = sys.argv[1:]
with tempfile.TemporaryDirectory() as directory:
    candidate_path = pathlib.Path(directory) / "99-warm.json"
    candidate = json.loads(pathlib.Path(baseline).read_text(encoding="utf-8"))
    sample = candidate["samples"][0]
    sample["rawSamplesMs"] = sample["rawSamplesMs"][:-1]
    sample["sampleCount"] = len(sample["rawSamplesMs"])
    sample["warmSamplesMs"] = sample["warmSamplesMs"][:-1]
    sample["warm"]["sampleCount"] = 99
    sample["warm"]["p50Ms"] = None
    sample["warm"]["p95Ms"] = None
    sample["warm"]["p99Ms"] = None
    sample["warm"]["reliable"] = {"p50": False, "p95": False, "p99": False}
    sample["warm"]["unavailableReason"] = {
        "p50": "insufficient_samples:99<100",
        "p95": "insufficient_samples:99<100",
        "p99": "insufficient_samples:99<100",
    }
    candidate_path.write_text(json.dumps(candidate), encoding="utf-8")
    result = subprocess.run([gate, baseline, str(candidate_path)], capture_output=True, text=True)
    if result.returncode == 0:
        raise SystemExit("P0 gate accepted 99 valid warm samples")
print("P0 gate rejected 99 valid warm samples")
PY
echo "P0 performance regression gate passed identity, evidence, and negative checks"
