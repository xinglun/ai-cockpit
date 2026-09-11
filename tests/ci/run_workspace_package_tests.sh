#!/usr/bin/env bash
set -uo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
metadata=""
cargo_bin=cargo
report="$root/target/workspace-package-coverage.json"
workers="${WORKSPACE_TEST_WORKERS:-2}"
test_threads="${WORKSPACE_TEST_THREADS:-4}"
while (($#)); do
  case "$1" in
    --metadata) metadata=${2:?}; shift 2 ;;
    --cargo) cargo_bin=${2:?}; shift 2 ;;
    --report) report=${2:?}; shift 2 ;;
    *) printf 'unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
done

if [[ ! "$workers" =~ ^[1-9][0-9]*$ ]]; then
  printf 'workspace test worker count must be a positive integer\n' >&2
  exit 2
fi
if [[ ! "$test_threads" =~ ^[1-9][0-9]*$ ]]; then
  printf 'workspace test thread count must be a positive integer\n' >&2
  exit 2
fi

tmp=$(mktemp -d "${TMPDIR:-/tmp}/workspace-packages.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
: >"$tmp/planned"
: >"$tmp/executed"
mkdir -p "$tmp/results"
state=passed
failure_phase=""
failed_package=""
failed_index=""
if [[ -z "$metadata" ]]; then
  metadata="$tmp/metadata.json"
  if ! (cd "$root" && "$cargo_bin" metadata --locked --format-version 1 --no-deps) >"$metadata"; then
    state=failed
    failure_phase=metadata
  fi
fi

if [[ "$state" == passed ]] && ! python3 - "$metadata" >"$tmp/planned" <<'PY'
import json
import sys

metadata = json.load(open(sys.argv[1], encoding="utf-8"))
packages = sorted({item["name"] for item in metadata["packages"] if item.get("source") is None})
if not packages:
    raise SystemExit("cargo metadata contains no workspace packages")
print("\n".join(packages))
PY
then
  state=failed
  failure_phase=metadata
  : >"$tmp/planned"
fi
if [[ "$state" == passed ]]; then
  packages=()
  while IFS= read -r package; do
    packages+=("$package")
  done <"$tmp/planned"
  pids=()
  active=0
  next=0
  package_count=${#packages[@]}
  while { [[ "$state" == passed ]] && ((next < package_count)) || ((active > 0)); }; do
    while [[ "$state" == passed ]] && ((next < package_count && active < workers)); do
      package=${packages[$next]}
      index=$next
      (
        set +e
        (cd "$root" && "$cargo_bin" test -p "$package" --all-targets -- --test-threads="$test_threads") \
          >"$tmp/results/$index.log" 2>&1
        result=$?
        printf '%s\n' "$result" >"$tmp/results/$index.status"
        exit 0
      ) &
      pids[$index]=$!
      ((next += 1))
      ((active += 1))
    done

    for index in "${!pids[@]}"; do
      if [[ -f "$tmp/results/$index.status" ]]; then
        wait "${pids[$index]}" 2>/dev/null
        result=$(<"$tmp/results/$index.status")
        if [[ "$result" != 0 && "$state" == passed ]]; then
          state=failed
          failure_phase=package_test
          failed_package=${packages[$index]}
          failed_index=$index
        fi
        unset 'pids[index]'
        ((active -= 1))
      fi
    done
    ((active > 0)) && sleep 0.05
  done
fi

if [[ "$state" != passed && -n "$failed_index" ]]; then
  cat "$tmp/results/$failed_index.log" >&2
fi

# Completion order is intentionally independent from the worker schedule so
# the report remains deterministic and can be compared across CI runs.
: >"$tmp/executed"
if [[ "$state" == passed || -n "$failed_index" ]]; then
  for index in "${!packages[@]}"; do
    if [[ -f "$tmp/results/$index.status" ]] && [[ "$(<"$tmp/results/$index.status")" == 0 ]]; then
      printf '%s\n' "${packages[$index]}" >>"$tmp/executed"
    fi
  done
fi

mkdir -p "$(dirname "$report")"
python3 - "$tmp/planned" "$tmp/executed" "$report" "$state" "$failure_phase" "$failed_package" <<'PY'
import json
import sys

def lines(path):
    return [line for line in open(path, encoding="utf-8").read().splitlines() if line]

planned = lines(sys.argv[1])
executed = lines(sys.argv[2])
report = {
    "executed": executed,
    "omitted": sorted(set(planned) - set(executed)),
    "planned": planned,
    "schemaVersion": 1,
    "state": sys.argv[4],
}
if sys.argv[5]:
    report["failurePhase"] = sys.argv[5]
if sys.argv[6]:
    report["failedPackage"] = sys.argv[6]
open(sys.argv[3], "w", encoding="utf-8").write(json.dumps(report, indent=2, sort_keys=True) + "\n")
PY

[[ "$state" == passed ]] || exit 1
printf 'workspace package coverage passed: %s packages\n' "$(wc -l <"$tmp/executed" | tr -d ' ')"
