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
route_readmes = [
    Path('docs/current/README.md'),
    Path('docs/current/README.zh-CN.md'),
    Path('docs/current/README.ja.md'),
    Path('docs/getting-started/README.md'),
    Path('docs/getting-started/README.zh-CN.md'),
    Path('docs/getting-started/README.ja.md'),
    Path('docs/features/README.md'),
    Path('docs/features/README.zh-CN.md'),
    Path('docs/features/README.ja.md'),
    Path('docs/operations/README.md'),
    Path('docs/operations/README.zh-CN.md'),
    Path('docs/operations/README.ja.md'),
]
for path in route_readmes:
    if not path.exists():
        missing.append(f'{path}: missing three-language reader route entry')

parity_statuses = {
    Path('docs/reference/reference-parity.md'): ('Implemented', 'Partial', 'Deferred', 'External boundary'),
    Path('docs/reference/reference-parity.zh-CN.md'): ('已实现', '部分实现', '延期', '外部边界'),
    Path('docs/reference/reference-parity.ja.md'): ('Implemented', 'Partial', 'Deferred', 'External boundary'),
}
for path, statuses in parity_statuses.items():
    if path.exists():
        text = path.read_text(encoding='utf-8')
        for status in statuses:
            if status not in text:
                missing.append(f'{path}: missing parity status {status}')

release_baselines = {
    Path('docs/release/distribution.md'): 'Complete adopter acceptance baseline: `x86_64-unknown-linux-gnu`',
    Path('docs/release/distribution.zh-CN.md'): '完整 adopter acceptance 基线为 `x86_64-unknown-linux-gnu`',
    Path('docs/release/distribution.ja.md'): '完全な adopter acceptance baseline は `x86_64-unknown-linux-gnu`',
}
for path, phrase in release_baselines.items():
    if phrase not in path.read_text(encoding='utf-8'):
        missing.append(f'{path}: missing explicit single-target acceptance baseline')

boundary_phrases = {
    Path('docs/capabilities.md'): 'human-facing projection',
    Path('docs/capabilities.zh-CN.md'): '面向人的 projection',
    Path('docs/capabilities.ja.md'): '人間向け projection',
    Path('docs/reference/outcome-report.md'): 'never silently translates or changes Contract bytes',
    Path('docs/reference/outcome-report.zh-CN.md'): '不会擅自翻译或改变 Contract bytes',
    Path('docs/reference/outcome-report.ja.md'): '勝手に翻訳・変更',
}
for path, phrase in boundary_phrases.items():
    if phrase not in path.read_text(encoding='utf-8'):
        missing.append(f'{path}: missing explicit boundary statement: {phrase}')

# Keep the user-facing command/capability inventory synchronized with the
# actual Runtime surfaces.  This is intentionally a small static contract:
# the MCP implementation remains the source of truth for the tool names, while
# the three language pages must expose every name and the CLI reference must
# expose every top-level read-only diagnostic entry.
mcp_docs = [Path('docs/capabilities.md'), Path('docs/capabilities.zh-CN.md'), Path('docs/capabilities.ja.md')]
for path in mcp_docs:
    text = path.read_text(encoding='utf-8')
    for tool in ('status', 'work_item_get', 'work_item_outcome', 'work_item_list',
                 'blockers', 'safe_actions', 'knowledge_query', 'evidence_get',
                 'delegated_evidence_list', 'repository_observe', 'preflight', 'verify',
                 'work_item_validate', 'work_item_parallel'):
        if f'`{tool}`' not in text:
            missing.append(f'{path}: MCP tool inventory omits {tool}')

for path in [Path('docs/reference/commands.md'), Path('docs/reference/commands.zh-CN.md'), Path('docs/reference/commands.ja.md')]:
    text = path.read_text(encoding='utf-8')
    for command in ('capability show', 'diagnose'):
        if f'`{command}`' not in text:
            missing.append(f'{path}: CLI command inventory omits {command}')

for path in [Path('docs/release/distribution.md'), Path('docs/release/distribution.zh-CN.md'), Path('docs/release/distribution.ja.md')]:
    text = path.read_text(encoding='utf-8')
    if 'x86_64-unknown-linux-gnu' not in text:
        missing.append(f'{path}: release acceptance baseline target is missing')
    acceptance_calls = re.findall(
        r'tests/release/adopter(?:_upgrade)?_acceptance\.sh[\s\S]*?--target\s+([^\s]+)',
        text,
    )
    if not acceptance_calls or any(target != 'x86_64-unknown-linux-gnu' for target in acceptance_calls):
        missing.append(f'{path}: acceptance example target must match the documented complete baseline')

for path in [Path('docs/reference/reference-parity.md'), Path('docs/reference/reference-parity.zh-CN.md'), Path('docs/reference/reference-parity.ja.md')]:
    text = path.read_text(encoding='utf-8')
    if 'humanHandoff' not in text or 'Implemented' not in text and '已实现' not in text:
        missing.append(f'{path}: human-facing MCP projection status is stale')

for phrase in ('WI-03 至 WI-38', 'WI-36 已在本地验收', 'WI-35 负责', 'internal progress plan', 'development checkout'):
    if phrase in public:
        missing.append(f'public documentation contains internal phrase: {phrase}')
if re.search(r'(?m)^\s*ai-cockpit mcp\s*$', public):
    missing.append('public documentation contains repository-less ai-cockpit mcp command')
if missing:
    raise SystemExit('\n'.join(missing))
print('documentation acceptance passed')
PY
