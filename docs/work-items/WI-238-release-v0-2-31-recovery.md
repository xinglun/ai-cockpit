---
author: AI Cockpit maintainers
title: "WI-238 — v0.2.31 release recovery"
workItemId: WI-238-release-v0-2-31-recovery
description: "Redeliver the v0.2.31 release from a clean default-branch Work Item after immutable WI-237 recovery."
audience:
  - maintainer
  - reviewer
  - adopter
status: implemented
authority: canonical
lastVerifiedBy: WI-238-release-v0-2-31-recovery
---

# WI-238 — v0.2.31 release recovery

WI-238 is the clean successor for WI-237. WI-237's immutable archive and
pre-merge finalization receipt are retained after hosted quality exposed a
missing tri-language parity binding and the attempted repair advanced an
unmerged head. This Work Item redelivers the same bounded release change from
the synchronized default branch.

## Acceptance boundary

- The release quality route tolerates a repository with no active Work Item
  directory, with a deterministic regression test.
- All three parity rows bind verification evidence and the pre-merge
  finalization receipt before hosted checks run.
- The failed immutable v0.2.30 tag is not rewritten or reused; v0.2.31 is
  published only from the reviewed merged head after hosted checks pass.
- Public v0.2.31 and N-1 upgrade acceptance use downloaded immutable artifacts
  only, with isolated roots and a cleaned temporary run root.

## Recovery boundary

WI-237 remains immutable historical recovery evidence. The successor is bound
through `.ai/decisions/WI-237-release-route-recovery-v0-2-31.recovery.json`.
No predecessor Contract, Summary, Outcome, Events, verification, archive, or
finalization receipt is rewritten.

## References

- [Reference parity ledger](../reference/reference-parity.md)
- [WI-237 immutable Work Item](WI-237-release-route-recovery-v0-2-31.md)
- [Release distribution](../release/distribution.md)
