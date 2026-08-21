---
author: AI Cockpit maintainers
title: "Work Items"
description: "The repository-local governed lifecycle for implementation work."
audience:
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - work_item_lifecycle
---

# Work Items

This repository is governed by the installed Rust `ai-cockpit` Runtime. It must
not install the V1 template. Every change uses the repository-local `.ai/`
Contract, evidence, and human decision records.

Each Work Item uses one branch, one base revision, one change scope, one evidence
bundle, and one outcome. A Work Item cannot claim completion from prose alone.

Required sections are Intent and Goal, Scope and Out of Scope, Sources and
Unknowns, Acceptance Criteria, Required Evidence, Base Revision, Changed Files,
Verification, Human Decisions, and Outcome. The English file has semantic
Chinese and Japanese equivalents whenever the Work Item is user-facing or
changes Runtime behavior.

## Runtime commands

Use the installed Runtime with an explicit repository context:

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

The Runtime is external and shared; `.ai/` is repository-local. There is no
global current repository or Work Item. Read `.ai/README.md` for the Agent
route and `.ai/glossary.md` for canonical terms.
