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

# Exercise the actual inline extraction used by the workflow, not a second unzip implementation.
python3 - "$root" <<'PY'
from pathlib import Path
import subprocess
import sys
import tempfile
import zipfile

workflow = (Path(sys.argv[1]) / '.github/workflows/ci.yml').read_text()
marker = '      - name: Extract the exact shared gate-plan files without deprecated dependencies\n'
block = workflow.split(marker, 1)[1].split('      - name:', 1)[0]
script = block.split("          python3 - <<'PY'\n", 1)[1].rsplit('          PY', 1)[0]
script = '\n'.join(line[10:] for line in script.splitlines()) + '\n'
expected = {'release/ai-cockpit': b'fixture executable', 'quality-selection.json': b'{}', 'quality-route.json': b'{}'}
for case in ['valid', 'foreign', 'duplicate', 'symlink']:
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        downloads = root / 'target/ci-gate-plan-download'
        downloads.mkdir(parents=True)
        with zipfile.ZipFile(downloads / 'artifact.zip', 'w') as archive:
            for name, data in expected.items():
                archive.writestr(name, data)
            if case == 'foreign':
                archive.writestr('../outside', b'foreign')
            elif case == 'duplicate':
                # Replacing an expected identity with a second path must fail without relying on warning text.
                archive.writestr('other/ai-cockpit', b'foreign')
            elif case == 'symlink':
                entry = zipfile.ZipInfo('untrusted-link')
                entry.create_system = 3
                entry.external_attr = 0o120777 << 16
                archive.writestr(entry, 'outside')
        result = subprocess.run([sys.executable, '-c', script], cwd=root, capture_output=True, text=True)
        assert (result.returncode == 0) == (case == 'valid'), (case, result.stdout, result.stderr)
        if case == 'valid':
            assert not result.stderr and not result.stdout
            for name, data in expected.items():
                assert (root / 'target' / name).read_bytes() == data
        else:
            assert not (root / 'target/release/ai-cockpit').exists(), 'invalid archive must be rejected before any payload install'
print('actual artifact extraction: exact payload and invalid identities passed')
PY
