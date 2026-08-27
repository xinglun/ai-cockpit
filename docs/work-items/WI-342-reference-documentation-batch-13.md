---
author: AI Cockpit maintainers
title: "WI-342 — reference documentation, distribution, and enterprise boundaries"
workItemId: WI-342-reference-documentation-batch-13
description: "Compare the next ten pinned reference paths one by one and record evidence-backed Rust counterparts without copying source history or wire formats."
audience: [maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-342-reference-documentation-batch-13
capabilityClaims:
  - reference_parity
---

# WI-342 — reference documentation, distribution, and enterprise boundaries

## Intent and boundary

This Work Item compares ten paths from the pinned reference commit
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` one file at a time. It records the
target's semantic responsibility and explicit different-design or reference-only
boundary; it does not copy reference Python, Make, source adopter records,
provider claims, or JSON wire formats.

The comparison covers distribution, documentation architecture and authority,
documentation context, enterprise controls, and external identity. It updates
only the comparison ledger, tri-language parity documentation, and this Work
Item's reader-facing record. Runtime behavior, release publication, adopter
acceptance, global Agent/MCP configuration, and later reference paths are out
of scope.

## File-by-file decision

The pinned paths and evidence-backed decisions are recorded in
`tests/conformance/reference_file_inventory.json` and the tri-language
`docs/reference/reference-file-comparison*` ledgers. Eight paths are
`implemented-different-by-design`; two source-specific control/context records
are `reference-only`. No path is silently treated as equivalent, deferred, or
missing.

The target inherits the object/adopter boundary: one shared Runtime, explicit
repository context, repository-local `.ai/` state, external provider evidence,
and no local claim of enterprise identity or compliance. Contract/source text
remains authoritative; localized presentation does not rewrite governance facts.

## Acceptance

- Every listed path is present exactly once in the pinned inventory with one
  evidence-backed classification and valid target counterparts or an explicit
  reference-only boundary.
- The English, Simplified Chinese, and Japanese comparison/parity pages state
  the same semantic, non-wire decision and current ledger counts.
- Source plan/context metadata and source adopter control observations are not
  copied into Runtime or treated as target evidence.
- Inventory, documentation, and repository gates pass without changing
  generated history or immutable evidence.

[简体中文](WI-342-reference-documentation-batch-13.zh-CN.md) ·
[日本語](WI-342-reference-documentation-batch-13.ja.md)
