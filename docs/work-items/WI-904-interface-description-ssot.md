---
author: AI Cockpit maintainers
title: "WI-904 — Work-item outcome interface description single source"
description: "Derive CLI, MCP, and reference interface facts from one protocol-owned description."
audience: [maintainer, reviewer]
status: in_progress
authority: explicit-user-authorization
workItemId: WI-904-interface-description-ssot
lastVerifiedBy: WI-904-interface-description-ssot
---

[简体中文](WI-904-interface-description-ssot.zh-CN.md) · [日本語](WI-904-interface-description-ssot.ja.md)

# WI-904 — Work-item outcome interface description single source

This Work Item removes duplicated interface facts from the work-item outcome
discovery path. The protocol-owned description remains the source for CLI,
MCP, and the marked reference projections; human explanations remain owned by
the reference pages.

## Acceptance

- CLI parsing, MCP validation, and generated reference facts agree with the
  protocol-owned description.
- The discovery path is deterministic and does not observe repository state or
  start verification processes.
- Outcome delivery, authorization, and lifecycle behavior are unchanged.
