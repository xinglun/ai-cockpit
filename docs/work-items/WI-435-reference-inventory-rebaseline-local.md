---
author: AI Cockpit maintainers
title: "WI-435 — local reference inventory rebaseline"
workItemId: WI-435-reference-inventory-rebaseline-local
description: "Rebind the file-level reference ledger to the maintained local semantic reference without silently promoting changed source decisions."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-435-reference-inventory-rebaseline-local
---

# WI-435 — local reference inventory rebaseline

This Work Item explicitly rebinds the file-level comparison ledger to the
maintainer-provided local checkout selected by `AI_COCKPIT_REFERENCE_ROOT`.
The source is pinned to commit
`fde3380f81fea5fd2e288f7a8849f737dc074060`; the public reference repository is
not required. This is an inventory and documentation change, not a semantic
comparison batch and not a source-content copy.

[简体中文](WI-435-reference-inventory-rebaseline-local.zh-CN.md) · [日本語](WI-435-reference-inventory-rebaseline-local.ja.md)

## Scope and safety boundary

- Record the current tracked path set (4,450 paths), 160 changed paths, and
  669 paths retired since the prior ledger.
- Preserve each prior decision as history; changed non-history records remain
  `deferred-next-batch` until a later file-by-file review.
- Keep the previous source commit and manifest digest recoverable, and make
  the machine ledger, lock file, tests, and tri-language documentation agree.
- Do not copy reference files, change Rust Runtime behavior, modify CI policy,
  or infer a governance decision from a source update.

The current ledger contains 3,681 generated-history, 223
implemented-different-by-design, one implemented-equivalent, four
not-applicable, 62 reference-only, and 479 deferred-next-batch records. The
retired path list is historical metadata, not a current parity claim.

## Verification boundary

The rebaseline is accepted only when the local-source policy, old-ledger
regression, current-ledger regression, documentation checks, parity checks,
and workspace tests pass. A changed or removed source path must remain visible
in the ledger; a missing checkout, moving commit, or public-network fallback
is a failure.
