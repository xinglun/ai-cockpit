---
author: AI Cockpit maintainers
title: "WI-550 — Lifecycle and Outcome script comparison batch 39"
description: "Compare sixteen pinned reference scripts and record Rust-native or external boundaries without copying source implementation."
audience:
  - maintainer
  - reviewer
  - adopter
status: in_progress
authority: canonical
workItemId: WI-550-reference-file-comparison-batch-39
lastVerifiedBy: WI-550-reference-file-comparison-batch-39
---

# WI-550 — Lifecycle and Outcome script comparison batch 39

## Objective

Read sixteen maintained reference scripts one by one at pinned local commit
`fde3380f81fea5fd2e288f7a8849f737dc074060`. Record semantic parity and
non-claims for the shared Rust Runtime and attached adopter repositories. This
batch does not copy Python modules, provider state, or source JSON wire formats.

## File-level result

The complete file-level mapping is maintained in
[`reference-file-comparison.md`](../reference/reference-file-comparison.md#wi-550--lifecycle-and-outcome-script-comparison-batch-39)
and in `tests/conformance/reference_file_inventory.json`. The sixteen records
are classified as fifteen `implemented-different-by-design` and one
`reference-only` provider-facing presentation boundary; no `migrate-gap` is
claimed.

## Adopter boundary

Attached repositories inherit the shared Runtime, explicit repository binding,
isolated Contract/evidence/knowledge, fail-closed lifecycle, and human Outcome
handoff. They do not inherit source Python registries, provider policy values,
or source wire formats.

## Acceptance

- The inventory records exactly sixteen current paths at the pinned source
  commit, with a non-empty reason and counterpart or explicit boundary.
- No selected path remains `deferred-next-batch` or `migrate-gap`; retired
  history remains append-only.
- English, Simplified Chinese, and Japanese comparison and parity pages state
  the same decisions and adopter boundary.
- Inventory, documentation, formatting, lint, and workspace verification checks
  pass before this Work Item is finished.
