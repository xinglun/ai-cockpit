---
author: AI Cockpit maintainers
title: "WI-240 — Documentation status and reference truth consistency"
workItemId: WI-240-doc-status-consistency
description: "Historical documentation-governance delivery recovered by WI-245 after its immutable PR failed hosted governance."
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-245-doc-status-parity-recovery
authority: canonical
---

# WI-240 — Documentation status and reference truth consistency

WI-240 produced a verified archive and canonical pre-merge finalization on an
older default-branch base, but PR #194 did not merge. Hosted governance exposed
an immutable failed-delivery boundary after later release, parity, and close
records advanced `main`. Its archived Contract, Summary, Outcome, events,
verification, and finalization bytes remain historical truth on the retained
predecessor branch and are not imported or rewritten by this documentation.

The Runtime-generated successor receipt
`.ai/decisions/WI-240-doc-status-consistency.recovery.json` binds those exact
predecessor digests and delegates the still-applicable status, inventory, and
release-truth delivery to WI-245 on `origin/main@87bfd866`.

## Recovery boundary

- PR #194 is closed as superseded and remains unmerged.
- WI-245 replays implementation content, not WI-240 lifecycle records.
- The pinned public reference commit remains unchanged.
- Existing provider, release, SBOM, parity, and terminal-decision truth from
  intervening Work Items is preserved.

## References

- [WI-245 successor](WI-245-doc-status-parity-recovery.md)
- [Reference file comparison](../reference/reference-file-comparison.md)
- [Reference source parity](../reference/reference-parity.md)
- [Release distribution](../release/distribution.md)
