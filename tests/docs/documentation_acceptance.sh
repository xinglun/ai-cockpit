#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
cd "$root"

python3 - <<'PY'
from pathlib import Path
import re

missing = []
for path in [Path('README.md'), Path('README.zh-CN.md'), Path('README.ja.md')]:
    text = path.read_text(encoding='utf-8')
    for target in re.findall(r'\[[^]]+\]\(([^)]+)\)', text):
        if target.startswith(('http://', 'https://', '#', 'mailto:')):
            continue
        target = target.split('#', 1)[0]
        if not (path.parent / target).exists():
            missing.append(f'{path}: {target}')
for path in Path('docs').rglob('*.md'):
    if path.parts[:2] == ('docs', 'superpowers'):
        continue
    text = path.read_text(encoding='utf-8')
    if text.startswith('---\n'):
        frontmatter = text.split('---\n', 2)[1]
        for key in ('author:', 'title:', 'description:', 'audience:', 'status:', 'authority:', 'lastVerifiedBy:'):
            if not any(line.startswith(key) for line in frontmatter.splitlines()):
                missing.append(f'{path}: missing {key}')
    for target in re.findall(r'\[[^]]+\]\(([^)]+)\)', text):
        if target.startswith(('http://', 'https://', '#', 'mailto:')):
            continue
        target = target.split('#', 1)[0]
        if not (path.parent / target).exists():
            missing.append(f'{path}: {target}')
public = '\n'.join(
    p.read_text(encoding='utf-8')
    for p in [Path('README.md'), Path('README.zh-CN.md'), Path('README.ja.md')]
    + [p for p in Path('docs').rglob('*.md') if p.parts[:2] != ('docs', 'superpowers') and p.parts[:2] != ('docs', 'work-items')]
)
for phrase in ('WI-03 至 WI-38', 'WI-36 已在本地验收', 'WI-35 负责', 'internal progress plan', 'development checkout'):
    if phrase in public:
        missing.append(f'public documentation contains internal phrase: {phrase}')
if missing:
    raise SystemExit('\n'.join(missing))
print('documentation acceptance passed')
PY
