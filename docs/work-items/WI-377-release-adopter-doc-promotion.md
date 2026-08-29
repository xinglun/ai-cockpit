---
author: AI Cockpit maintainers
title: "WI-377 — post-close documentation promotion recovery"
description: "Promote the WI-376 tri-language documentation after its verified close and make the required post-close check explicit."
workItemId: WI-377-release-adopter-doc-promotion
audience: [maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-377-release-adopter-doc-promotion
terminalArchive: .ai/work-items/archive/WI-377-release-adopter-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-377-release-adopter-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-377-release-adopter-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-377-release-adopter-doc-promotion.close.json
capabilityClaims: [documentation_governance, release_quality]
---

# WI-377 — post-close documentation promotion recovery

[简体中文](WI-377-release-adopter-doc-promotion.zh-CN.md) · [日本語](WI-377-release-adopter-doc-promotion.ja.md)

## Intent

Restore the post-close documentation projection required by the repository
quality gates. The Runtime and immutable v0.2.39 release/adopter evidence are
unchanged.

## Scope and boundary

- Promote the WI-376 Work Item and reference-parity projections with the
  deterministic `promote_closed_work_item.py` helper.
- Record the required post-close check in the inherited Agent route so future
  releases do not leave `completed` documentation on a closed Work Item.
- Keep all Runtime, release, and historical evidence bytes unchanged.

## Result

WI-376 is represented as `implemented` in all three languages and each
projection is bound to its archive, verification, finalization, and close
receipts. The post-close promotion check is now an explicit delivery step.
