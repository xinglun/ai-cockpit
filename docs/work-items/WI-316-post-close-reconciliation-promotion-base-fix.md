---
author: AI Cockpit maintainers
title: "WI-316 — post-close reconciliation promotion base fix"
workItemId: WI-316-post-close-reconciliation-promotion-base-fix
description: "Rebind the recovered promotion correction to the latest remote default base without rewriting W315 history."
audience:
  - maintainer
  - reviewer
status: recovered
authority: canonical
lastVerifiedBy: WI-316-post-close-reconciliation-promotion-base-fix
---

# WI-316 — post-close reconciliation promotion base fix

## Intent and boundary

W315 is an immutable archived delivery. Hosted CI rejected its Contract before
evaluation because `baseRevision` pointed at an older branch head. This bounded
successor starts from the latest `origin/main`, binds the actual CI base, and
redelivers the reviewed W314/W315 correction without rewriting history.

## Scope and acceptance

- The Contract records the exact latest remote default revision used by hosted CI.
- Valid successor/supersede recovery remains a historical promotion exception;
  retry, malformed, and foreign recovery remain fail-closed.
- W315 archive and all predecessor evidence remain byte-for-byte immutable.
- English, Simplified Chinese, and Japanese Work Item/parity projections are
  synchronized before verification.

## Verification

Run promotion/documentation regressions, `cargo fmt`, warning-denied clippy, the
locked workspace test suite, and hosted CI on this exact reviewed branch. The
installed Runtime remains the governance interface.

## Related history

- W315: immutable delivery rejected by hosted base-revision gate.
- W316: bounded successor with the corrected remote base binding.

[简体中文](WI-316-post-close-reconciliation-promotion-base-fix.zh-CN.md) ·
[日本語](WI-316-post-close-reconciliation-promotion-base-fix.ja.md)
