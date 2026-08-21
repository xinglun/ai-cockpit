#!/usr/bin/env bash
set -euo pipefail

# Final replacement acceptance for the Rust Runtime. This is intentionally a
# source-repository acceptance harness; it never builds or runs a local
# fallback binary. The runtime identity is always read from the installed
# `ai-cockpit` executable and every repository-bound call carries --repo.

usage() {
  echo "usage: $0 --repo <repository> [--output <directory>] [--runtime <binary>]" >&2
  exit 2
}

repo=""
output=""
runtime="${AI_COCKPIT_BIN:-ai-cockpit}"
while (($# > 0)); do
  case "$1" in
    --repo) repo=${2:?missing --repo value}; shift 2 ;;
    --output) output=${2:?missing --output value}; shift 2 ;;
    --runtime) runtime=${2:?missing --runtime value}; shift 2 ;;
    -h|--help) usage ;;
    *) usage ;;
  esac
done

[[ -n "$repo" ]] || usage
repo=$(cd "$repo" && pwd -P)
root=$(cd "$(dirname "$0")/../.." && pwd -P)
[[ -d "$repo/.git" || -f "$repo/.git" ]] || { echo "repository is not a Git checkout: $repo" >&2; exit 1; }
command -v "$runtime" >/dev/null 2>&1 || { echo "installed runtime not found: $runtime" >&2; exit 1; }

if [[ -z "$output" ]]; then
  output="$root/target/final-replacement-acceptance"
fi
mkdir -p "$output"
output=$(cd "$output" && pwd -P)
rm -f "$output"/*.json "$output"/*.txt "$output"/SHA256SUMS

hash_file() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    sha256sum "$1" | awk '{print $1}'
  fi
}

runtime_version=$($runtime --version)
runtime_version=${runtime_version#ai-cockpit }
runtime_digest=$(hash_file "$(command -v "$runtime")")
runtime_path=$(command -v "$runtime")
repository_id=$("$runtime" status --repo "$repo" | python3 -c 'import json,sys; print(json.load(sys.stdin)["repositoryId"])')
reference_commit=$(sed -n 's/^commit = "\([0-9a-f]\{40\}\)"$/\1/p' "$root/tests/conformance/v1-reference.lock")
[[ "$reference_commit" =~ ^[0-9a-f]{40}$ ]] || { echo "invalid V1 reference lock" >&2; exit 1; }

tmp=$(mktemp -d "${TMPDIR:-/tmp}/ai-cockpit-final-replacement.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
reference_root="$tmp/v1-reference"
git init -q "$reference_root"
git -C "$reference_root" fetch --depth=1 \
  https://github.com/spirex-ds-dev/ai-cockpit-template.git "$reference_commit"
git -C "$reference_root" checkout -q --detach FETCH_HEAD
[[ "$(git -C "$reference_root" rev-parse HEAD)" == "$reference_commit" ]] || {
  echo "fetched V1 reference does not match the lock" >&2
  exit 1
}

printf '%s\n' \
  "runtime=$runtime_path" \
  "version=$runtime_version" \
  "digest=sha256:$runtime_digest" \
  "repositoryId=$repository_id" \
  "v1ReferenceCommit=$reference_commit" > "$output/runtime.txt"

steps_file="$tmp/steps.tsv"
: > "$steps_file"
overall=passed
run_step() {
  local name=$1
  shift
  local safe_name=${name//[^A-Za-z0-9_.-]/_}
  local log="$output/$safe_name.log"
  if "$@" >"$log" 2>&1; then
    printf '%s\tpassed\t%s\n' "$name" "" >> "$steps_file"
  else
    printf '%s\tfailed\t%s\n' "$name" "$(tail -n 1 "$log" | tr '\t' ' ')" >> "$steps_file"
    overall=failed
  fi
}

run_step conformance-corpus \
  cargo test -p cockpit-core --test conformance -- --test-threads=1
run_step adversarial-corpus \
  cargo test -p cockpit-core --test adversarial_v2 -- --test-threads=1
run_step performance-regression \
  bash "$root/tests/performance/regression_gate_test.sh"
run_step release-workflow-policy \
  bash "$root/tests/release/workflow_policy.sh" "$root/.github/workflows/release.yml"
run_step outcome-dialog-test \
  cargo test -p cockpit-cli --test intelligence -- --test-threads=1
run_step locked-v1-oracle \
  env AI_COCKPIT_V1_ROOT="$reference_root" cargo test -p cockpit-core --test v1_oracle -- --ignored --test-threads=1

tracked_copy=$(git -C "$root" ls-files | grep -E '(^|/)(Makefile\.ai|ai_cockpit/|installer/|runtime/)' || true)
if [[ -n "$tracked_copy" ]]; then
  printf '%s\n' "$tracked_copy" > "$output/copied-v1-runtime.txt"
  printf '%s\tfailed\ttracked V1 runtime paths detected\n' no-copied-v1-runtime >> "$steps_file"
  overall=failed
else
  printf '%s\tpassed\t\n' no-copied-v1-runtime >> "$steps_file"
fi

python3 - "$output" "$steps_file" "$runtime_version" "$runtime_digest" "$repository_id" "$reference_commit" "$overall" <<'PY'
import json
import pathlib
import sys
from datetime import datetime, timezone

out, steps_path, version, digest, repository_id, reference_commit, overall = sys.argv[1:]
steps = []
for line in pathlib.Path(steps_path).read_text(encoding="utf-8").splitlines():
    name, status, reason = line.split("\t", 2)
    steps.append({"name": name, "status": status, "reason": reason or None})
payload = {
    "schemaVersion": 1,
    "acceptance": "final_replacement",
    "state": overall,
    "recordedAt": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
    "runtimeVersion": version,
    "runtimeDigest": "sha256:" + digest,
    "repositoryId": repository_id,
    "v1ReferenceCommit": reference_commit,
    "steps": steps,
}
pathlib.Path(out, "acceptance.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
PY

(
  cd "$output"
  while IFS= read -r file; do
    hash_file "$file" | awk -v path="${file#./}" '{print $1 "  " path}'
  done < <(find . -type f ! -name SHA256SUMS -print | sort)
) > "$output/SHA256SUMS"

if [[ "$overall" != passed ]]; then
  echo "final replacement acceptance failed; see $output/acceptance.json" >&2
  exit 1
fi
echo "final replacement acceptance passed: $output"
