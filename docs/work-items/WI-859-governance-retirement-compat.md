---
author: AI Cockpit maintainers
title: "WI-859 — governance retirement compatibility"
description: "Align the static governance integrity gate with the Runtime retirement protocol."
audience: [maintainer, reviewer]
status: implemented
authority: authorized
workItemId: WI-859-governance-retirement-compat
lastVerifiedBy: WI-859-governance-retirement-compat
terminalArchive: .ai/work-items/archive/WI-859-governance-retirement-compat.contract.json
terminalVerification: .ai/evidence/WI-859-governance-retirement-compat.verification.json
terminalDecision: .ai/decisions/WI-859-governance-retirement-compat.close.json
---

[简体中文](WI-859-governance-retirement-compat.zh-CN.md) · [日本語](WI-859-governance-retirement-compat.ja.md)

# WI-859 — governance retirement compatibility

This Work Item makes the governance integrity gate understand the Runtime's
append-only retirement route. A valid retirement receipt and matching retired
archive are terminal historical cleanup evidence; they do not claim a new
verification result or manufacture a close decision. Invalid, foreign, or
digest-mismatched retirement records remain fail-closed.

The change is limited to the gate, its regression fixtures, and the required
tri-language documentation projection.
