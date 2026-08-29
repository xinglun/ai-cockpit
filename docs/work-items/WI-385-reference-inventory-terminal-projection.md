---
author: AI Cockpit maintainers
title: "WI-385 — reference inventory terminal projection"
workItemId: WI-385-reference-inventory-terminal-projection
description: "Complete the post-close terminal projection for WI-384 without rewriting immutable history."
audience: [maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-385-reference-inventory-terminal-projection
terminalArchive: .ai/work-items/archive/WI-385-reference-inventory-terminal-projection.contract.json
terminalVerification: .ai/evidence/WI-385-reference-inventory-terminal-projection.verification.json
terminalFinalization: .ai/decisions/WI-385-reference-inventory-terminal-projection.finalize.5000ae21b509964497aa74cb0abb6463b1c0737042b05ae6d130044eed153358.json
terminalDecision: .ai/decisions/WI-385-reference-inventory-terminal-projection.close.json
---

# WI-385 — reference inventory terminal projection

## Intent and boundary

WI-385 is the explicit successor for a documentation consistency defect found
after WI-384 closed. It changes only the three-language parity row and the
three-language WI-384 status metadata; WI-384 archive, evidence, finalization,
close, and recovery records remain immutable.

## Acceptance

- The parity ledgers mark WI-384 `Implemented` and link terminal records.
- The WI-384 documents use `implemented` status and bind archive, verification,
  finalization, and close records.
- Documentation and governance integrity gates pass without Runtime or
  predecessor-byte changes.

[简体中文](WI-385-reference-inventory-terminal-projection.zh-CN.md) · [日本語](WI-385-reference-inventory-terminal-projection.ja.md)
