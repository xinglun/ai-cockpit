---
author: AI Cockpit maintainers
title: "WI-490 — WI-489 terminal documentation projection"
description: "Promote the bounded WI-489 documentation projection and terminate the post-close documentation gate."
audience: [maintainer, reviewer, adopter]
workItemId: WI-490-wi489-terminal-doc-promotion
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-490-wi489-terminal-doc-promotion
---

# WI-490 — WI-489 terminal documentation projection

This bounded documentation Work Item promotes the WI-489 tri-language pages
and parity registration using WI-489's immutable terminal evidence. It exists
to close the documentation projection loop without changing Runtime behavior,
historical evidence, or global Agent/MCP configuration.

[简体中文](WI-490-wi489-terminal-doc-promotion.zh-CN.md) · [日本語](WI-490-wi489-terminal-doc-promotion.ja.md)

## Scope

- Promote the three WI-489 Work Item pages to terminal evidence-backed metadata.
- Promote the three WI-489 parity rows with archive, verification, finalization,
  and close references.
- Keep this Work Item's own pages and parity registration inside the same
  bounded projection so the terminal checker has no recursive successor.

## Acceptance

- The six WI-489 projection pages/rows are promoted without changing authored
  content or immutable governance records.
- The post-close promotion and status-consistency checks recognize this exact
  documentation-only scope as self-terminal.
- English, Chinese, and Japanese documentation checks pass, and no global
  Agent/MCP configuration is changed.

## Verification

- `bash tests/docs/promote_closed_work_item_test.sh`
- `bash tests/docs/work_item_status_consistency_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
- `python3 tests/conformance/reference_file_inventory.py --check`
