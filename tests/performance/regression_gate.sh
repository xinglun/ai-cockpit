#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <baseline.json> <candidate.json>" >&2
  exit 2
fi

# This gate consumes captured evidence only. It never builds or runs a local
# source fallback and it cannot authorize a candidate whose runtime or
# repository identity differs from the declared baseline.
python3 - "$1" "$2" <<'PY'
import json
import pathlib
import sys

baseline_path = pathlib.Path(sys.argv[1])
candidate_path = pathlib.Path(sys.argv[2])

def load(path):
    try:
        with path.open(encoding="utf-8") as handle:
            value = json.load(handle)
    except (OSError, json.JSONDecodeError) as error:
        raise SystemExit(f"performance evidence unreadable: {path}: {error}")
    if not isinstance(value, dict):
        raise SystemExit(f"performance evidence must be an object: {path}")
    return value

baseline = load(baseline_path)
candidate = load(candidate_path)
required = ("schemaVersion", "runtimeVersion", "runtimeDigest", "repositoryId", "samples", "budgets")
for name, value in (("baseline", baseline), ("candidate", candidate)):
    missing = [field for field in required if field not in value]
    if missing:
        raise SystemExit(f"{name} missing required fields: {', '.join(missing)}")
    def valid_digest(digest):
        suffix = str(digest).removeprefix("sha256:")
        return str(digest).startswith("sha256:") and len(suffix) == 64 and all(char in "0123456789abcdefABCDEF" for char in suffix)
    if value["schemaVersion"] != 1:
        raise SystemExit(f"{name} has unsupported schemaVersion")
    if not valid_digest(value["runtimeDigest"]):
        raise SystemExit(f"{name} has invalid runtimeDigest")
    if not valid_digest(value["repositoryId"]):
        raise SystemExit(f"{name} has invalid repositoryId")

for field in ("runtimeVersion", "runtimeDigest", "repositoryId"):
    if candidate[field] != baseline[field]:
        raise SystemExit(f"identity mismatch for {field}")

samples = {}
for sample in candidate["samples"]:
    if not isinstance(sample, dict) or not str(sample.get("name", "")).strip():
        raise SystemExit("candidate has malformed sample")
    name = sample["name"]
    if name in samples:
        raise SystemExit(f"candidate has duplicate sample: {name}")
    samples[name] = sample
failures = []
for budget in baseline["budgets"]:
    name = budget.get("name")
    sample = samples.get(name)
    if sample is None:
        failures.append(f"sample_missing:{name}")
        continue
    elapsed = sample.get("elapsedMs")
    iterations = sample.get("iterations")
    limit = budget.get("maxElapsedMs")
    if not isinstance(elapsed, int) or not isinstance(iterations, int) or not isinstance(limit, int):
        failures.append(f"sample_malformed:{name}")
    elif iterations <= 0:
        failures.append(f"iterations_zero:{name}")
    elif elapsed > limit:
        failures.append(f"budget_exceeded:{name}:{elapsed}>{limit}")

if failures:
    print(json.dumps({"state": "failed", "failures": failures}, sort_keys=True))
    raise SystemExit(1)
print(json.dumps({"state": "passed", "budgets": len(baseline["budgets"])}, sort_keys=True))
PY
