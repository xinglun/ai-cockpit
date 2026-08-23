#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
policy="$root/tests/workflow/resource_finalization_policy.sh"

[[ -x "$policy" ]] || chmod +x "$policy"

# Positive repository check.
"$policy" "$root"

# Regression: removing one language-neutral transition must fail the gate. The
# fixture is isolated and disposable; no repository policy file is mutated.
fixture=$(mktemp -d "${TMPDIR:-/tmp}/ai-cockpit-finalization-policy.XXXXXX")
cleanup() { rm -rf "$fixture"; }
trap cleanup EXIT

mkdir -p "$fixture/docs/reference" "$fixture/docs/work-items"
for path in \
  docs/reference/agent-workflow.md \
  docs/reference/agent-workflow.ja.md \
  docs/reference/agent-workflow.zh-CN.md \
  docs/reference/reference-parity.md \
  docs/reference/reference-parity.ja.md \
  docs/reference/reference-parity.zh-CN.md \
  docs/work-items/WI-160-resource-finalization-baseline.md \
  docs/work-items/WI-160-resource-finalization-baseline.ja.md \
  docs/work-items/WI-160-resource-finalization-baseline.zh-CN.md \
  docs/work-items/WI-161-historical-runtime-close.md \
  docs/work-items/WI-161-historical-runtime-close.ja.md \
  docs/work-items/WI-161-historical-runtime-close.zh-CN.md; do
  mkdir -p "$fixture/$(dirname "$path")"
  cp "$root/$path" "$fixture/$path"
done

python3 - "$fixture/docs/reference/agent-workflow.zh-CN.md" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = text.replace("finalize-verify", "verification-step-removed")
path.write_text(text, encoding="utf-8")
PY

if "$policy" "$fixture" >/dev/null 2>&1; then
  printf 'resource finalization policy regression did not fail after removing finalize-verify\n' >&2
  exit 1
fi

printf 'resource finalization policy tests passed\n'
