#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 <runtime-binary> <repo> <output.json> [warm-samples]"
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi
if [[ $# -lt 3 || $# -gt 4 ]]; then
  usage >&2
  exit 2
fi

binary=$1
repo=$2
output=$3
warm_samples=${4:-20}

if [[ ! -f "$binary" || -L "$binary" || ! -x "$binary" ]]; then
  echo "runtime binary must be an executable regular file (no symlink)" >&2
  exit 1
fi
if [[ ! -d "$repo" ]]; then
  echo "repository directory does not exist" >&2
  exit 1
fi
if ! [[ "$warm_samples" =~ ^[1-9][0-9]*$ ]]; then
  echo "warm sample count must be a positive integer" >&2
  exit 2
fi

script_dir=$(cd "$(dirname "$0")" && pwd -P)
export PYTHONPATH="$script_dir${PYTHONPATH:+:$PYTHONPATH}"
exec python3 "$script_dir/knowledge_query_benchmark.py" \
  "$binary" "$repo" "$output" "$warm_samples"
