#!/usr/bin/env bash
# Reuse the canonical, source-local cheap consumers before expensive work.
set -euo pipefail

repository=$(cd "$(dirname "$0")/../.." && pwd -P)
seen_repo=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      if [[ "$seen_repo" == true || $# -lt 2 || -z "$2" ]]; then
        printf 'exactly one nonempty --repo is allowed\n' >&2; exit 2
      fi
      repository=$2; seen_repo=true; shift 2 ;;
    *) printf 'unsupported repository precheck argument: %s\n' "$1" >&2; exit 2 ;;
  esac
done
repository=$(cd "$repository" && pwd -P)
if [[ $(git -C "$repository" rev-parse --is-inside-work-tree) != true \
   || -n $(git -C "$repository" rev-parse --show-prefix) ]]; then
  printf 'prechecks require the source Git worktree root\n' >&2; exit 2
fi
cd "$repository"
for path in tests/docs/governance_cost_baseline_test.py \
            tests/ci/workflow_convergence_test.sh \
            tests/release/action_runtime_policy_test.sh; do
  if [[ ! -f "$path" || -L "$path" ]]; then
    printf 'missing or unsafe canonical precheck: %s\n' "$path" >&2; exit 2
  fi
done
python3 tests/docs/governance_cost_baseline_test.py
bash tests/ci/workflow_convergence_test.sh
bash tests/release/action_runtime_policy_test.sh
