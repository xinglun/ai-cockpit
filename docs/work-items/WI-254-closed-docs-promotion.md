---
author: AI Cockpit maintainers
title: "WI-254 — Deterministic closed documentation promotion"
workItemId: WI-254-closed-docs-promotion
description: "Promote controlled Work Item documentation fields from exact immutable close evidence and make the check a required quality gate."
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-254-closed-docs-promotion
authority: canonical
---

# WI-254 — Deterministic closed documentation promotion

WI-254 is the Runtime-recorded successor to WI-253. Its recovery receipt binds
WI-253's canonical Contract, Summary, Outcome, events, archive, verification,
sequence-2 finalization, and close evidence. Those lifecycle records remain
immutable.

## Acceptance boundary

- `tests/docs/promote_closed_work_item.py` validates the exact repository and
  Work Item identity, archive Contract raw digest, passing verification,
  linear finalization chain, sequence-2 deleted receipt, merge identity, and
  structured approved close before planning documentation changes.
- The write boundary is limited to `status`, `lastVerifiedBy`, and the four
  `terminal*` frontmatter fields in the exact three Work Item documents, plus
  the one exact row in each reference-parity document. Contract-language body
  prose and all `.ai` lifecycle truth are not rewritten.
- `--check-all` is a mandatory documentation/quality gate for governed closed
  Work Items. Invalid identity or filesystem input fails before a document is
  written; stale canonical projections fail the check.
- The same helper promotes WI-253 in this change and WI-254 from the detached,
  synchronized default-branch closure context after WI-254 closes.

## Lifecycle handoff

The complete delivery sequence is `close → promote closed docs → terminal CI`.
The helper is an explicit repository workflow command; Runtime Core does not
claim to edit Markdown automatically. A green pre-close PR run therefore does
not replace the required terminal projection and terminal default-branch run.

## References

- [WI-253 predecessor](WI-253-docs-terminalization.md)
- [Agent workflow](../reference/agent-workflow.md)
- [Reference parity](../reference/reference-parity.md)
