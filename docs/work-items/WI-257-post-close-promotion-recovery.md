---
author: AI Cockpit maintainers
title: "WI-257 — Post-close promotion recovery"
workItemId: WI-257-post-close-promotion-recovery
description: "Recover typed post-close documentation promotion from a clean current base without rewriting the failed predecessor."
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-257-post-close-promotion-recovery
authority: canonical
---

# WI-257 — Post-close promotion recovery

WI-257 re-delivers the repository-owned post-close documentation orchestrator
from the current default-branch base. WI-256 and closed PR #208 remain external
immutable failed-delivery history: this Work Item neither imports their `.ai`
records nor represents them as repository terminal truth.

## Acceptance boundary

- A typed plan binds repository identity, exact synchronized `origin/main`, an
  approved close, sequence-2 finalization, archive/evidence identities, and the
  six controlled documentation paths with exact before/after digests.
- Planning and apply fail closed for stale or descendant revisions, foreign or
  malformed identity, duplicate or unknown JSON fields, symlink/nonregular
  input or output, dirty or partial projection, and unexpected paths. Applying
  an already-current plan is a deterministic no-op.
- The isolated bare-origin regression advertises `main` through `HEAD`, so a
  clone exercises the same default-branch identity used by the orchestrator.
- WI-255's three Work Item and three reference-parity projections become
  `Implemented` without modifying any immutable `.ai` lifecycle byte.

## Lifecycle handoff

The repository workflow is:

```text
close → visible Outcome → post-close plan/apply → check-all → terminal CI
```

WI-257 remains conditional until its own verified close. Its future terminal
paths are the archived Contract, verification evidence, finalization chain,
and close receipt named in the parity ledger; their existence is never claimed
before Runtime creates them.

## References

- [Agent workflow](../reference/agent-workflow.md)
- [Commands](../reference/commands.md)
- [Reference parity](../reference/reference-parity.md)
- [Failed predecessor PR #208](https://github.com/xinglun/ai-cockpit/pull/208)
