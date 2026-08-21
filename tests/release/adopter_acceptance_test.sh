#!/usr/bin/env bash
set -euo pipefail

script="$(cd "$(dirname "$0")" && pwd)/adopter_acceptance.sh"
repo="$(git rev-parse --show-toplevel)"

bash -n "$script"
grep -q -- '--repository OWNER/REPOSITORY' "$script"
grep -q -- 'releasePublished' "$script"
grep -q -- 'first-adopter-smoke' "$script"
grep -q -- 'nodesReused' "$script"
grep -q -- 'SHA256SUMS' "$script"
if grep -Eq 'cargo[[:space:]]+(build|run)' "$script"; then
  printf 'acceptance harness must not obtain Runtime through cargo build/run\n' >&2
  exit 1
fi

invalid_output="$(mktemp -d)"
if "$script" --repository xinglun/ai-cockpit --tag v0.1.1 --target unsupported --output "$invalid_output" --source-repo "$repo" >/dev/null 2>&1; then
  printf 'unsupported targets must fail closed\n' >&2
  exit 1
fi

run_public=''
if run_public="$(printenv AI_COCKPIT_RUN_PUBLIC_ACCEPTANCE)"; then :; fi
if [[ "$run_public" == 1 ]]; then
  target=''
  if target="$(printenv AI_COCKPIT_ACCEPTANCE_TARGET)"; then :; fi
  if [[ -z "$target" ]]; then
    case "$(uname -s):$(uname -m)" in
      Darwin:arm64) target=aarch64-apple-darwin ;;
      Darwin:x86_64) target=x86_64-apple-darwin ;;
      Linux:x86_64) target=x86_64-unknown-linux-gnu ;;
      *) printf 'set AI_COCKPIT_ACCEPTANCE_TARGET for this host\n' >&2; exit 1 ;;
    esac
  fi
  output="$(mktemp -d)"
  "$script" --repository xinglun/ai-cockpit --tag v0.1.1 --target "$target" --output "$output" --source-repo "$repo"
  jq -e '.adopterAcceptance == "passed" and .releasePublished == true and ([.steps[] | select(.state != "passed")] | length == 0)' "$output/acceptance.json" >/dev/null
  jq -e --arg version v0.1.1 '.version == $version and .releasePublished == true and (.binaryDigest | startswith("sha256:"))' "$output/runtime.json" >/dev/null
  jq -e '.state == "not_ready" and .intent == "" and (.scope | length == 0) and .authority == "unknown"' "$output/work-items/first-adopter-smoke.contract.json" >/dev/null
  (cd "$output" && shasum -a 256 -c SHA256SUMS >/dev/null)
fi

printf 'adopter acceptance harness checks passed\n'
