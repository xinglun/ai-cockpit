---
author: AI Cockpit maintainers
title: "WI-818 — evidence Contract repair"
description: "Make required evidence classes fail early while preserving historical Contract compatibility."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized
workItemId: WI-818-evidence-contract-repair
lastVerifiedBy: WI-818-evidence-contract-repair
terminalArchive: .ai/work-items/archive/WI-818-evidence-contract-repair.contract.json
terminalVerification: .ai/evidence/WI-818-evidence-contract-repair.verification.json
terminalDecision: .ai/decisions/WI-818-evidence-contract-repair.close.json
---

[简体中文](WI-818-evidence-contract-repair.zh-CN.md) · [日本語](WI-818-evidence-contract-repair.ja.md)

# WI-818 — evidence Contract repair

## Intent and boundary

This Work Item makes required evidence classes explicit before verification,
keeps all declared classes visible in diagnostics, and preserves readable
historical Contracts. The Runtime-generated archive, verification evidence, and
close decision are the authority; this page does not rewrite or recreate them.

## Verification

The recorded verification receipt passed the declared evidence-class and
historical-compatibility checks. Future Work Items must use the supported
evidence vocabulary before they begin expensive verification.
