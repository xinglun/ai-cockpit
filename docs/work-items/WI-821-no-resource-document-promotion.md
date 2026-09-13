---
author: AI Cockpit maintainers
title: "WI-821 — no-resource document promotion"
description: "Keep post-close documentation promotion strict without fabricating provider finalization for local Work Items."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized
workItemId: WI-821-no-resource-document-promotion
lastVerifiedBy: WI-821-no-resource-document-promotion
terminalArchive: .ai/work-items/archive/WI-821-no-resource-document-promotion.contract.json
terminalVerification: .ai/evidence/WI-821-no-resource-document-promotion.verification.json
terminalDecision: .ai/decisions/WI-821-no-resource-document-promotion.close.json
---

[简体中文](WI-821-no-resource-document-promotion.zh-CN.md) · [日本語](WI-821-no-resource-document-promotion.ja.md)

# WI-821 — no-resource document promotion

## Intent and boundary

This Work Item makes the documentation promotion helper distinguish an explicit
no-resource Contract from a provider-bound Contract. Local Work Items may close
without a fabricated finalization receipt; provider-bound Work Items continue to
require the complete identity-bound finalization chain.

The WI-819 close record and projection repair are included only to restore the
post-release governance boundary. Archived WI-819 evidence is not rewritten.

## Verification

Focused tests cover absent and null `resourceContext`, missing resource-bound
finalization, deterministic no-resource terminal references, and the no-write
check path.
