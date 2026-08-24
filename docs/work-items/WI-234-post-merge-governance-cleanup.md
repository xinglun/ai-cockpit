---
author: AI Cockpit maintainers
title: "WI-234 — Post-merge governance cleanup and stale-close prevention"
workItemId: WI-234-post-merge-governance-cleanup
description: "Close the post-merge governance loop, prevent stale merged receipts, and start the next batch from a clean environment."
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-234-post-merge-governance-cleanup
---

# WI-234 — Post-merge governance cleanup and stale-close prevention

## Intent

Close the post-merge governance loop before the next comparison batch. Preserve
immutable failure and recovery history, then remove obsolete branches,
worktrees, and temporary checkouts so the next batch starts from a clean
environment.

## Why this Work Item exists

Recent hosted failures exposed two recurring process gaps:

- a reviewed head could already be on the synchronized default branch while
  the pre-merge finalization receipt still said `unmerged`;
- post-merge generated evidence could fall outside the original release
  Contract scope, making an otherwise real merge impossible to close without
  a recovery Work Item.

These are fixed as workflow controls. Existing failed PRs, Contracts, and
evidence remain immutable; cleanup records their disposition instead of
rewriting history.

## Scope

- Add the deterministic `stale_awaiting_merge_close` governance-gate finding
  and regression fixture.
- Update the English, Chinese, and Japanese parity ledgers with the final
  disposition of WI-222, WI-227, WI-230, and this Work Item.
- Preserve WI-230's append-only historical transitions and bind its recovery
  through the current Work Item. WI-222 remains linked immutable history; no
  synthetic second successor edge is created.
- Archive exact local dirty worktree bytes and branch tips outside the
  repository, then remove only the obsolete WI-189/WI-193/WI-222/WI-223/
  WI-224/WI-225/WI-228 checkouts and refs after their PR dispositions are
  recorded.

## Out of scope

No predecessor Contract, Summary, Outcome, Events, verification, archive, or
hosted failure bytes are rewritten. No global Agent/MCP configuration or
user-owned root changes are made.

## Acceptance

1. A current-release stale pre-merge receipt is rejected when its reviewed
   head is present on the synchronized default branch, with the stable
   `stale_awaiting_merge_close` finding.
2. The parity ledgers agree in all three languages and reference the exact
   evidence and decision paths.
3. Historical branches/worktrees are either preserved in a digest-bound
   external archive or removed exactly after their PR is closed/superseded.
4. The root worktree's pre-existing user files are unchanged.
5. Installed Runtime inspection and the declared governance/documentation
   checks pass before the Work Item is finalized.

## Recovery and cleanup policy

An immutable predecessor with a real successor is linked once, using the
Runtime's supported predecessor edge. If a second edge would be required, the
history remains linked in documentation and a new independent Work Item is
used; no receipt is fabricated. Cleanup is fail-closed: archive status,
untracked bytes, branch tips, PR state, and SHA-256 manifest are captured
before a worktree or branch is removed.

## References

- [Reference parity ledger](../reference/reference-parity.md)
- [Chinese parity ledger](../reference/reference-parity.zh-CN.md)
- [Japanese parity ledger](../reference/reference-parity.ja.md)
- [Repository governance gate](../../tests/ci/governance_integrity_gate.py)
- [Gate regression](../../tests/ci/governance_integrity_gate_test.sh)
