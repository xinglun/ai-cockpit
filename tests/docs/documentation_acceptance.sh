#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd -P)
cd "$root"

python3 - <<'PY'
from pathlib import Path
import json
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
    work_item_match = re.match(
        r'^WI-(\d+)([A-Za-z]?)(?:-[A-Za-z0-9-]+)?(?:\.(?:zh-CN|ja))?\.md$',
        path.name,
    ) if path.parts[:2] == ('docs', 'work-items') else None
    requires_work_item_frontmatter = bool(
        work_item_match and int(work_item_match.group(1)) >= 180
    )
    if requires_work_item_frontmatter and not text.startswith('---\n'):
        missing.append(f'{path}: missing required Work Item frontmatter')
    if text.startswith('---\n'):
        frontmatter_parts = text.split('---\n', 2)
        if len(frontmatter_parts) != 3:
            missing.append(f'{path}: malformed frontmatter delimiters')
            continue
        frontmatter = frontmatter_parts[1]
        for key in ('author:', 'title:', 'description:', 'audience:', 'status:', 'authority:', 'lastVerifiedBy:'):
            if not any(line.startswith(key) for line in frontmatter.splitlines()):
                missing.append(f'{path}: missing {key}')
        if requires_work_item_frontmatter:
            expected_id = re.sub(r'\.(?:zh-CN|ja)\.md$|\.md$', '', path.name)
            work_item_lines = [
                line.split(':', 1)[1].strip()
                for line in frontmatter.splitlines()
                if line.startswith('workItemId:')
            ]
            if work_item_lines != [expected_id]:
                missing.append(
                    f'{path}: workItemId must be exactly {expected_id}'
                )
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

for path in [Path('README.md'), Path('README.zh-CN.md'), Path('README.ja.md')]:
    text = path.read_text(encoding='utf-8')
    for marker in ('start', 'preflight', 'checkpoint', 'verify', 'finish', 'archive', 'close'):
        if marker not in text:
            missing.append(f'{path}: lifecycle route omits {marker}')

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

comparison_documents = (
    Path('docs/reference/reference-file-comparison.md'),
    Path('docs/reference/reference-file-comparison.zh-CN.md'),
    Path('docs/reference/reference-file-comparison.ja.md'),
)
comparison_markers = (
    '87bfd86645adf7f4a6f86e447763542988371039',
    'ai-cockpit 0.2.31',
    '1064f61154168149aebb63a4ad15374d50fc729c8699142c7a193c22eb6fb8f9',
    '720',
    '.ai/project/adopter-capability-manifest.json',
    '.ai/project/capabilities.json',
    '.ai/project/success_criteria.json',
    '.ai/project_profile.yaml',
)
for path in comparison_documents:
    text = path.read_text(encoding='utf-8')
    for marker in comparison_markers:
        if marker not in text:
            missing.append(f'{path}: current comparison baseline omits {marker}')

release_baselines = {
    Path('docs/release/distribution.md'): 'Persisted adopter acceptance baseline: `aarch64-apple-darwin`',
    Path('docs/release/distribution.zh-CN.md'): '持久化 adopter acceptance 基线：`aarch64-apple-darwin`',
    Path('docs/release/distribution.ja.md'): '永続化された adopter acceptance baseline: `aarch64-apple-darwin`',
}
for path, phrase in release_baselines.items():
    if phrase not in path.read_text(encoding='utf-8'):
        missing.append(f'{path}: missing persisted single-target acceptance baseline')

release_receipt = json.loads(
    Path('.ai/evidence/WI-239-release-v0-2-31-adopter-acceptance/acceptance.json')
    .read_text(encoding='utf-8')
)
release_provider = json.loads(
    Path('.ai/evidence/WI-239-release-v0-2-31-adopter-acceptance/release.json')
    .read_text(encoding='utf-8')
)
persisted_target = release_receipt.get('target')
if persisted_target != 'aarch64-apple-darwin':
    missing.append('WI-239 persisted adopter target is not aarch64-apple-darwin')
if release_provider.get('immutable') is not False:
    missing.append('WI-239 provider Release truth no longer reports immutable=false')
for path in release_baselines:
    text = path.read_text(encoding='utf-8')
    if '`immutable: false`' not in text:
        missing.append(f'{path}: provider immutable=false truth is missing')
    if '32696048024' not in text or 'x86_64-unknown-linux-gnu' not in text:
        missing.append(f'{path}: hosted Linux acceptance evidence boundary is missing')
    if 'public immutable `v0.2.31`' in text or 'immutable public `v0.2.31`' in text:
        missing.append(f'{path}: mutable provider Release is described as immutable')

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
    for tool in ('status', 'work_item_get', 'work_item_outcome', 'work_item_status', 'work_item_list',
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
    if persisted_target not in text:
        missing.append(f'{path}: persisted release acceptance baseline target is missing')
    acceptance_calls = re.findall(
        r'tests/release/adopter(?:_upgrade)?_acceptance\.sh[\s\S]*?--target\s+([^\s]+)',
        text,
    )
    if not acceptance_calls or any(target != persisted_target for target in acceptance_calls):
        missing.append(f'{path}: acceptance example target must match the persisted baseline')

for path in [Path('docs/reference/reference-parity.md'), Path('docs/reference/reference-parity.zh-CN.md'), Path('docs/reference/reference-parity.ja.md')]:
    text = path.read_text(encoding='utf-8')
    if 'humanHandoff' not in text or 'Implemented' not in text and '已实现' not in text:
        missing.append(f'{path}: human-facing MCP projection status is stale')
    parity_status = '已实现' if path.name.endswith('.zh-CN.md') else 'Implemented'
    implemented_ids = []
    for work_item_doc in sorted(Path('docs/work-items').glob('WI-*.md')):
        if work_item_doc.name.endswith(('.zh-CN.md', '.ja.md')):
            continue
        work_item_text = work_item_doc.read_text(encoding='utf-8')
        work_item_id = re.search(r'^workItemId:\s*(WI-[0-9]+(?:-[A-Za-z0-9-]+)?)\s*$', work_item_text, re.MULTILINE)
        status = re.search(r'^status:\s*implemented\s*$', work_item_text, re.MULTILINE)
        if work_item_id and status:
            implemented_ids.append(work_item_id.group(1))
    latest_implemented = max(implemented_ids, key=lambda value: int(value.split('-')[1])) if implemented_ids else None
    for work_item in ('WI-121', 'WI-122', 'WI-123', 'WI-125', 'WI-126') + ((latest_implemented,) if latest_implemented else ()):
        if work_item not in text:
            missing.append(f'{path}: current implementation baseline omits {work_item}')
    if parity_status not in text:
        missing.append(f'{path}: current implementation baseline omits {parity_status}')
    for marker in (
        'WI-245',
        'v0.2.31',
        'aarch64-apple-darwin',
        '32696048024',
        'x86_64-unknown-linux-gnu',
        '`immutable: false`',
    ):
        if marker not in text:
            missing.append(f'{path}: current release/reference truth omits {marker}')

for path in [Path('docs/operations/README.md'), Path('docs/operations/README.zh-CN.md'), Path('docs/operations/README.ja.md')]:
    text = path.read_text(encoding='utf-8')
    if 'x86_64-unknown-linux-gnu' not in text:
        missing.append(f'{path}: current adopter baseline target is missing')
    if re.search(r'v[0-9]+\.[0-9]+\.[0-9]+', text):
        missing.append(f'{path}: operations baseline must not hard-code a release version')

for path in [Path('docs/reference/contract-fields.md'), Path('docs/reference/contract-fields.zh-CN.md'), Path('docs/reference/contract-fields.ja.md')]:
    text = path.read_text(encoding='utf-8')
    for status in ('Implemented', 'Partial', 'External'):
        if status not in text:
            missing.append(f'{path}: Contract/Summary field mapping omits {status}')
    for section in ('Contract', 'Summary'):
        if section not in text:
            missing.append(f'{path}: Contract/Summary field mapping omits {section}')

for phrase in ('WI-03 至 WI-38', 'WI-36 已在本地验收', 'WI-35 负责', 'internal progress plan', 'development checkout'):
    if phrase in public:
        missing.append(f'public documentation contains internal phrase: {phrase}')
if re.search(r'(?m)^\s*ai-cockpit mcp\s*$', public):
    missing.append('public documentation contains repository-less ai-cockpit mcp command')
if missing:
    raise SystemExit('\n'.join(missing))
print('documentation acceptance passed')
PY

python3 tests/docs/work_item_status_consistency.py \
  --repo "${AI_COCKPIT_STATUS_DOCS_REPO:-$root}"

bash tests/docs/getting_started_semantic.sh
