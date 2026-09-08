---
author: AI Cockpit maintainers
title: "WI-700 — WI-698 base revalidation"
description: "Preserve the immutable WI-698 delivery while binding its revalidation to the then-current default base."
audience: [contributor, maintainer, reviewer]
workItemId: WI-700-wi698-base-revalidation
status: recovered
authority: human:repository-owner
lastVerifiedBy: WI-700-wi698-base-revalidation
---

[简体中文](WI-700-wi698-base-revalidation.zh-CN.md) · [日本語](WI-700-wi698-base-revalidation.ja.md)

# WI-700 — WI-698 base revalidation

WI-700 is an immutable recovery predecessor for WI-698. Its Contract and
verification were bound to an older default branch, so WI-701 revalidates the
same bounded delivery from the current default base without rewriting WI-698 or
WI-700 archive and evidence bytes.

## Recovery boundary

- Archive: `.ai/work-items/archive/WI-700-wi698-base-revalidation.contract.json`
- Historical verification: `.ai/evidence/WI-700-wi698-base-revalidation.verification.json`
- Recovery decision: `.ai/decisions/WI-700-wi698-base-revalidation.recovery.json`
- Successor: WI-701-wi700-current-base-revalidation

No new implementation semantics, governance rules, protocol formats, or
cross-agent branch changes are introduced by this recovery projection.
