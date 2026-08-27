---
author: AI Cockpit maintainers
title: "WI-337 — reference documentation foundation retry"
workItemId: WI-337-reference-docs-foundation-retry
description: "Redeliver the first five pinned reference governance-documentation comparisons through a clean successor lifecycle while preserving WI-336 history."
audience:
  - maintainer
  - reviewer
status: recovered
authority: canonical
lastVerifiedBy: WI-337-reference-docs-foundation-retry
---

# WI-337 — reference documentation foundation retry

## Intent and recovery boundary

WI-337 is an explicit successor to WI-336. The predecessor remains immutable
history because Runtime 0.2.33 could not reconcile its pre-verification Contract
amendment chain. This retry reuses the same five pinned paths and does not add
new implementation scope.

## Reused comparison

The file-level classifications, Rust counterparts, and non-wire boundary are
recorded in [the WI-336 comparison](WI-336-reference-docs-foundation.md) and
the tri-language ledgers. WI-337 revalidates those bytes against the current
repository and reviewed PR context; it does not copy source Python, Make,
provider, or historical tooling.

## Acceptance and verification

1. The five pinned paths retain exactly one explicit inventory classification,
   counterpart, and non-overclaiming reason.
2. English, Simplified Chinese, and Japanese comparison/parity ledgers agree.
3. The planned GitHub resource context is bound before verification and the
   current Runtime evidence is repository- and snapshot-bound.
4. Inventory, documentation, parity, and locked workspace verification pass.

Predecessor recovery: `.ai/decisions/WI-336-reference-docs-foundation.recovery.e7ccd6381b1492fd0ba72be8c7305217748f03d9c7509a7c58db693e8ba13261.json`.

[简体中文](WI-337-reference-docs-foundation-retry.zh-CN.md) ·
[日本語](WI-337-reference-docs-foundation-retry.ja.md)
