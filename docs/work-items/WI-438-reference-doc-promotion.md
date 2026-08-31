---
author: AI Cockpit maintainers
title: "WI-438 — closed WI-437 documentation promotion"
workItemId: WI-438-reference-doc-promotion
description: "Promote the tri-language projections for the closed WI-437 governance rebaseline."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-438-reference-doc-promotion
---

# WI-438 — closed WI-437 documentation promotion

This documentation-only Work Item runs the repository-owned promotion helper
for WI-437 after its reviewed merge, finalization, and close. It keeps the
maintained local reference checkout at
`/Users/sei-rinn/dev/workspace_python/ai-cockpit-template` as the semantic
reference and does not access the public reference repository or change Runtime
behavior.

[简体中文](WI-438-reference-doc-promotion.zh-CN.md) · [日本語](WI-438-reference-doc-promotion.ja.md)

## Scope

- Promote WI-437's three Work Item documents and three reference-parity rows.
- Keep this Work Item's three language documents and pre-archive parity row
  current so the documentation gate can audit its own lifecycle.
- Do not rewrite immutable archive, verification, finalization, or close bytes.

## Verification

`tests/docs/promote_closed_work_item.py --work-item
WI-437-reference-rebaseline-governance` and `--check-all`, documentation
acceptance, parity/status checks, governance integrity, and the declared Runtime
verification must pass. Changes must remain limited to this Contract's scope.
