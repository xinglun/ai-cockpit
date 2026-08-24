---
author: AI Cockpit maintainers
title: "WI-235 — Finalization recovery and clean-batch boundary"
workItemId: WI-235-finalization-recovery
description: "Recover the archived WI-234 delivery while binding reviewed PR context before verification and archive."
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-235-finalization-recovery
---

# WI-235 — Finalization recovery and clean-batch boundary

WI-235 is a narrow successor for the process defect discovered in PR #185:
WI-234 was archived before `finalize-plan`, so the archived Contract retained a
pending resource context and the governance gate correctly rejected the missing
terminal decision. The failed PR and all WI-234 bytes remain immutable.

This successor binds the real reviewed PR context before verification, records
the recovery decision, and completes the normal finalization boundary. It also
proves that the next batch starts with no obsolete WI-234/WI-235 worktree or
branch left behind.

## Acceptance boundary

- `stale_awaiting_merge_close` regression remains fail closed.
- WI-234 is referenced as recovered through its exact recovery receipt.
- `finalize-plan` precedes verify, finish, and archive.
- Parallel attach migration fixtures use collision-resistant paths, so the full
  workspace test suite remains deterministic under concurrent test execution.
- The pre-merge finalization receipt, hosted checks, merge observation, exact
  cleanup, and structured close are all bound to the same PR head.

## References

- [Reference parity ledger](../reference/reference-parity.md)
- [WI-234 immutable Work Item](WI-234-post-merge-governance-cleanup.md)
- [Governance gate](../../tests/ci/governance_integrity_gate.py)
