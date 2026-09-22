---
author: AI Cockpit maintainers
title: "WI-993 — WI-867 terminal documentation projection"
description: "Complete the corrected terminal documentation projection for WI-867 after its self-terminal check remained stale."
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: authorized
workItemId: WI-993-wi867-doc-promotion
lastVerifiedBy: WI-993-wi867-doc-promotion
---

[简体中文](WI-993-wi867-doc-promotion.zh-CN.md) · [日本語](WI-993-wi867-doc-promotion.ja.md)

# WI-993 — WI-867 terminal documentation projection

WI-993 completes the bounded terminal projection for WI-867. It preserves
WI-867's archive, verification, and close evidence and changes only the
tri-language documentation projection and parity ledgers.

## Boundary

Only WI-867's three language pages, this Work Item's three language pages,
the three reference-parity ledgers, and Runtime-generated `.ai/` evidence are
in scope. Source behavior, release behavior, and historical governance bytes
are out of scope.

## Acceptance

- WI-867's three language pages and parity rows agree with its immutable terminal evidence.
- WI-993 has a bounded tri-language self-projection until close.
- The single-item WI-867 projection, repository-wide `--check-all`, parity, and status-consistency checks pass.
