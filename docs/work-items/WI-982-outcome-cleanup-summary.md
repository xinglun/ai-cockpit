---
author: AI Cockpit maintainers
title: "WI-982 — Outcome cleanup summary"
description: "Expose validated cleanup counts, exact resource identities, retention reasons, evidence references, and the next action in the shared human Outcome body."
audience: [maintainer, reviewer, contributor]
status: implemented
authority: human:user
workItemId: WI-982-outcome-cleanup-summary
lastVerifiedBy: WI-982-outcome-cleanup-summary
terminalArchive: .ai/work-items/archive/WI-982-outcome-cleanup-summary.contract.json
terminalVerification: .ai/evidence/WI-982-outcome-cleanup-summary.verification.json
terminalDecision: .ai/decisions/WI-982-outcome-cleanup-summary.close.json
---

[简体中文](WI-982-outcome-cleanup-summary.zh-CN.md) · [日本語](WI-982-outcome-cleanup-summary.ja.md)

# WI-982 — Outcome cleanup summary

This Work Item improves the human-facing cleanup summary produced from a
validated resource-finalization receipt. The shared projection reports deleted,
retained, and unknown counts; exact pull request, branch, and worktree
identities; the receipt-bound reason and evidence references; and the existing
executable next action. CLI and MCP continue to consume the same complete body,
and unknown provider display confirmation remains unknown.

Legacy finalization JSON stays readable through additive optional fields. This
Work Item does not change authorization, verification, release, or provider
display semantics.
