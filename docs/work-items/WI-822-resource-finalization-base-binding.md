---
author: AI Cockpit maintainers
title: "WI-822 — resource finalization base binding"
description: "Bind provider finalization to the actual reviewed PR base and preserve close recovery."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized
workItemId: WI-822-resource-finalization-base-binding
lastVerifiedBy: WI-822-resource-finalization-base-binding
terminalArchive: .ai/work-items/archive/WI-822-resource-finalization-base-binding.contract.json
terminalVerification: .ai/evidence/WI-822-resource-finalization-base-binding.verification.json
terminalFinalization: .ai/decisions/WI-822-resource-finalization-base-binding.finalize.json
terminalDecision: .ai/decisions/WI-822-resource-finalization-base-binding.close.json
---

[简体中文](WI-822-resource-finalization-base-binding.zh-CN.md) · [日本語](WI-822-resource-finalization-base-binding.ja.md)

# WI-822 — resource finalization base binding

## Intent and boundary

WI-822 separates the Contract base, the reviewed PR comparison base, and the
published execution identity. It records provider cleanup only after the
merged PR, exact branch, and exact worktree state have been verified. The
predecessor archive and its historical evidence remain unchanged.

## Verification

The formal Runtime verification passed all 12 planned workspace nodes. The
provider finalization receipt records the merged PR and exact branch/worktree
cleanup, and the close decision binds that finalization head.
