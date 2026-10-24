#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/../.." && pwd -P)
python3 - "$root" <<'PY'
from pathlib import Path
import subprocess
import sys

script = Path(sys.argv[1]) / 'tests/ci/run_no_warnings.py'
def run(code):
    return subprocess.run([sys.executable, str(script), '--', sys.executable, '-c', code], text=True, capture_output=True)

positive = run("print('warning: unused import: `File`')")
assert positive.returncode != 0, 'a real compiler warning with exit 0 must fail the quality boundary'
assert 'unused import' in positive.stdout + positive.stderr, 'the original diagnostic must remain visible'
for line in ['warning[E001]: diagnostic', 'WARNING: tool diagnostic', '(node:12) [DEP0005] DeprecationWarning: Buffer()', '::warning file=source.rs::diagnostic']:
    result = run('print(' + repr(line) + ')')
    assert result.returncode != 0, (line, result.stdout, result.stderr)
for line in ['test warning_parser_rejects_error ... ok', 'business error handled correctly', '{"fixture":"warning: unused"}', "quoted fixture: 'warning: unused'", '0 warnings found']:
    result = run('print(' + repr(line) + ')')
    assert result.returncode == 0, (line, result.stdout, result.stderr)
failed = run('import sys; print("actual command failed"); sys.exit(7)')
assert failed.returncode == 7 and 'actual command failed' in failed.stdout
large = run('print("x" * 200000); print("warning: emitted after a large successful stream")')
assert large.returncode != 0 and len(large.stdout) > 200000
for marker in ['[diagnostic truncated]', '[output truncated]']:
    assert run('print(' + repr(marker) + ')').returncode != 0, 'incomplete capture cannot certify zero warnings'
print('no-warning quality boundary: compiler/tool diagnostics, complete output, and negative fixtures passed')
PY
