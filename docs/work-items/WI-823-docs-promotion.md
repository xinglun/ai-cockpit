---
author: AI Cockpit maintainers
title: "WI-823 — terminal documentation projection repair"
description: "Restore evidence-bound tri-language documentation for recently closed Work Items."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized
workItemId: WI-823-docs-promotion
lastVerifiedBy: WI-823-docs-promotion
terminalArchive: .ai/work-items/archive/WI-823-docs-promotion.contract.json
terminalVerification: .ai/evidence/WI-823-docs-promotion.verification.json
terminalDecision: .ai/decisions/WI-823-docs-promotion.close.json
---

[简体中文](WI-823-docs-promotion.zh-CN.md) · [日本語](WI-823-docs-promotion.ja.md)

# WI-823 — terminal documentation projection repair

## Intent and boundary

This bounded documentation Work Item restores missing reader-facing pages for
WI-818, WI-820, and WI-822 and registers its own three-language projection.
Runtime behavior, release artifacts, object repositories, and immutable
Contract, verification, archive, recovery, finalization, and close bytes are
out of scope.

## Acceptance

- Each repaired Work Item has truthful English, Simplified Chinese, and Japanese pages.
- Each page and parity row points to the exact Runtime-owned evidence paths.
- Repository-wide documentation promotion and status checks pass without rewriting evidence.
