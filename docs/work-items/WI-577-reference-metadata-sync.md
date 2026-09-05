---
author: AI Cockpit maintainers
title: "WI-577 — current reference-comparison metadata synchronization"
description: "Keep the live comparison baseline and tri-language metadata projection bound to the reviewed release."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-577-reference-metadata-sync
lastVerifiedBy: WI-577-reference-metadata-sync
---

[简体中文](WI-577-reference-metadata-sync.zh-CN.md) · [日本語](WI-577-reference-metadata-sync.ja.md)

# WI-577 — current reference-comparison metadata synchronization

## Objective

Keep the reader-facing reference comparison and parity routes synchronized with
the pinned local reference source, the reviewed Rust baseline, and the
published Runtime identity. A small checked-in metadata sidecar is the single
source for these live facts and for current ledger counts.

## Scope and boundary

The scope is the six tri-language reference pages, the metadata sidecar, the
executable metadata regression test, its documentation-acceptance hook, and
these three-language Work Item pages. Historical batch paragraphs and
generated governance evidence remain append-only. Runtime behavior, object
repositories, global Agent/MCP configuration, and source implementation copies
are out of scope.

## Acceptance

- All six reference pages expose the same current source commit, metadata
  sidecar, and `lastVerifiedBy` value.
- The live Rust baseline and published Runtime version/digest match the
  sidecar; inventory counts are derived and checked rather than hand-counted.
- A stale header, count, source lock, or translated page fails closed in CI.
- No semantic classification, historical evidence, or object repository is
  rewritten.

## Verification

See the active Contract and `tests/docs/reference_comparison_metadata_test.py`.
The bounded checks include the reference inventory, documentation acceptance,
Work Item status consistency, and `git diff --check`.
