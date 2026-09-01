---
author: AI Cockpit maintainers
title: "WI-314 — finalization reconciliation redelivery"
workItemId: WI-314-finalization-reconciliation-redelivery
description: "Redeliver cleanup-before-close and append-only finalization reconciliation after an immutable hosted-quality failure."
audience:
  - maintainer
  - reviewer
status: recovered
authority: canonical
lastVerifiedBy: WI-314-finalization-reconciliation-redelivery
---

# WI-314 — finalization reconciliation redelivery

## Intent and boundary

WI-312 is preserved as immutable historical delivery. Its retained provider
finalization and conditional parity projection exposed an ordering defect after
merge. The first Runtime correction was delivered by WI-313, but PR #277 was
correctly rejected by hosted documentation gates before merge. This successor
redelivers the bounded correction from the synchronized default branch and
records the explicit W312 recovery without rewriting either predecessor.

## Scope and acceptance

- New Work Items cannot close while their provider finalization is retained,
  blocked, or unknown; only an identity-bound deleted result satisfies close.
- A legacy closed record may receive one append-only deleted transition only
  when the immutable predecessor, repository, Runtime, sequence, and exact
  cleanup postconditions all match.
- W312 is shown as `Recovered`, with its original Contract, evidence, archive,
  finalization, and close bytes unchanged. Conditional terminal parity rows
  without a valid recovery or reconciliation binding remain failures.
- English, Simplified Chinese, and Japanese parity/work-item projections are
  synchronized before verification and retain the exact evidence links.

## Verification

Run the focused finalization and documentation regressions, then `cargo fmt`,
clippy with warnings denied, and the locked full workspace test suite. Hosted
CI must pass on the exact reviewed branch before merge. The installed Runtime
is the governance interface; source builds are not a release-acceptance
substitute.

## Related history

- W312: immutable merged delivery recovered by this successor.
- W313 / PR #277: immutable failed hosted delivery; its branch and archive are
  retained as external audit history and are not revived.

[简体中文](WI-314-finalization-reconciliation-redelivery.zh-CN.md) ·
[日本語](WI-314-finalization-reconciliation-redelivery.ja.md)
