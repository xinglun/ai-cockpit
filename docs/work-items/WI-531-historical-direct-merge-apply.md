---
workItemId: WI-531-historical-direct-merge-apply
title: "WI-531 — bundled historical direct-merge application"
status: in_progress
mode: code
author: AI Cockpit maintainers
description: "Keep a real bundled merge parent and the immutable Contract base as separate, auditable facts."
audience:
  - maintainer
  - adopter
authority: canonical
lastVerifiedBy: documentation-acceptance
---

# WI-531 — bundled historical direct-merge application

## Intent and boundary

Make the published Runtime usable for an adopter whose older Work Items were
merged together without Pull Requests. This Work Item adds an optional
`historical.contractBaseRevision` field, keeps `pullRequest.baseRevision` bound
to the real merge commit's first parent, reports resource-context mismatch
categories, and proves the read-only plan can be completed without editing
historical bytes. No object repository is modified and no PR is invented.

## Acceptance

- A complete plan receipt is accepted as the first canonical direct-merge
  record, including when the bundled merge parent differs from the Contract
  base.
- Missing or foreign Contract-base binding, context, repository, Work Item,
  Git-parent, and runtime facts remain fail-closed with actionable fields.
- The plan and command documentation explain deterministic facts versus
  human-owned fields in English, Chinese, and Japanese.
- Protocol/repository tests cover preserved legacy context, generated
  historical context, bundled merge base drift, malformed input, and no-write
  rejection.

## Compatibility

The new field is optional and defaults to absent, so existing receipts whose
merge base already equals the Contract base remain readable. A mismatched base
is accepted only for `direct_merge_no_pr`/`historical_low` receipts that bind
the exact archived Contract digest and explicitly carry the Contract base.

## Object-repository handoff

`/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator` is read-only
for this Work Item. After the release, its team should rerun
`finalize-recovery-plan --merge-commit <sha>`, preserve both base fields, and
apply only the generated receipt. If a named `resourceContext.<field>` error
remains, report that field and do not edit `.ai/` records by hand.
