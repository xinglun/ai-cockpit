#!/usr/bin/env bash
set -uo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
metadata=""
cargo_bin=cargo
report="$root/target/workspace-package-coverage.json"
while (($#)); do
  case "$1" in
    --metadata) metadata=${2:?}; shift 2 ;;
    --cargo) cargo_bin=${2:?}; shift 2 ;;
    --report) report=${2:?}; shift 2 ;;
    *) printf 'unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
done

tmp=$(mktemp -d "${TMPDIR:-/tmp}/workspace-packages.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
: >"$tmp/planned"
: >"$tmp/executed"
state=passed
failure_phase=""
failed_package=""
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
  while IFS= read -r package; do
    if ! (cd "$root" && "$cargo_bin" test -p "$package" --all-targets -- --test-threads=1); then
      state=failed
      failure_phase=package_test
      failed_package=$package
      break
    fi
    printf '%s\n' "$package" >>"$tmp/executed"
  done <"$tmp/planned"
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
