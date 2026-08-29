---
author: AI Cockpit maintainers
title: "WI-375 — WI-374 terminal documentation promotion"
description: "Prepare the tri-language Work Item and parity projections for deterministic post-close promotion."
workItemId: WI-375-wi374-doc-status
audience: [maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-375-wi374-doc-status
terminalArchive: .ai/work-items/archive/WI-375-wi374-doc-status.contract.json
terminalVerification: .ai/evidence/WI-375-wi374-doc-status.verification.json
terminalFinalization: .ai/decisions/WI-375-wi374-doc-status.finalize.json
terminalDecision: .ai/decisions/WI-375-wi374-doc-status.close.json
capabilityClaims: [documentation_governance]
---

# WI-375 — WI-374 terminal documentation promotion

[简体中文](WI-375-wi374-doc-status.zh-CN.md) · [日本語](WI-375-wi374-doc-status.ja.md)

## Intent

Keep the closed WI-374 documentation and parity ledgers truthful through the
repository's explicit post-close promotion helper. This Work Item prepares and
verifies only the repository-local documentation boundary; the helper performs
the exact machine-owned terminal projection after close.

## Scope and boundary

- Maintain the three WI-374 language projections and three parity ledgers in the
  pre-close form required by the promotion helper.
- Verify documentation, parity, and governance integrity gates.
- Preserve immutable WI-374 Runtime evidence and use the documented
  `close → promote closed docs → terminal CI` sequence.

Runtime behavior, release assets, historical evidence bytes, and global Agent
or MCP configuration are outside this Work Item.

## Acceptance

1. WI-374 three-language documents and parity rows are valid pre-promotion
   projections and link the immutable terminal receipts.
2. Documentation, parity, and governance integrity checks pass before close.
3. After reviewed merge and close, the promotion helper can deterministically
   write only the terminal frontmatter and parity rows.
4. The reviewed Work Item is merged, finalized, closed, and cleaned exactly.

## Verification boundary

The Runtime records this Work Item's Contract, checkpoint, verification,
archive, finalization, and close evidence. `promote_closed_work_item.py` is the
explicit post-close documentation projection; it does not rewrite Runtime
truth or historical evidence.
