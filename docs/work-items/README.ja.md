---
author: AI Cockpit maintainers
title: "Work Items"
description: "この repository の repository-local governed implementation lifecycle。"
audience:
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: wi-42-documentation-governance
capabilityClaims:
  - work_item_lifecycle
---

# Work Items

この repository は install 済みの Rust `ai-cockpit` Runtime で governance します。V1
template は install しません。各変更は repository-local `.ai/` Contract、evidence、
human decision record を使います。

各 Work Item は branch、base revision、change scope、evidence bundle、outcome を一つずつ
持ち、文章だけで完了を宣言できません。Intent、Goal、Scope、Out of Scope、Sources、Unknowns、
Acceptance Criteria、Required Evidence、Base Revision、Changed Files、Verification、Human
Decisions、Outcome が必須です。adopter 向け、または Runtime behavior を変更する Work Item
は English、中文、日本語を同期します。

## Runtime commands

共有された外部 Runtime を使い、常に repository context を明示します。

```bash
ai-cockpit status --repo /path/to/ai-cockpit
ai-cockpit start --repo /path/to/ai-cockpit --id <id> \
  --intent "..." --goal "..." --scope "..." --authority authorized
ai-cockpit preflight --repo /path/to/ai-cockpit \
  --contract .ai/work-items/active/<id>.contract.json
ai-cockpit checkpoint --repo /path/to/ai-cockpit --id <id>
ai-cockpit verify --repo /path/to/ai-cockpit --work-item <id>
ai-cockpit finish --repo /path/to/ai-cockpit --id <id>
ai-cockpit archive --repo /path/to/ai-cockpit --id <id>
ai-cockpit close --repo /path/to/ai-cockpit --id <id> --human-decision approved
```

Runtime は外部で共有され、`.ai/` は repository-local です。global な current repository や
Work Item はありません。Agent route は `.ai/README.md`、canonical term は
`.ai/glossary.md` を参照してください。
