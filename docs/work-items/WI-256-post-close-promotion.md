---
author: AI Cockpit maintainers
title: "WI-256 — Typed post-close documentation promotion"
workItemId: WI-256-post-close-promotion
description: "Make post-close documentation promotion reproducible, identity-bound, and fail-closed."
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-256-post-close-promotion
authority: canonical
---

# WI-256 — Typed post-close documentation promotion

WI-256 closes the workflow gap exposed after WI-255: structural close was
valid, but the controlled tri-language documentation promotion was left to an
easy-to-forget manual command. This Work Item adds a repository-owned typed
plan/apply wrapper. It does not move Markdown behavior into Runtime Core and
does not rewrite any immutable `.ai` lifecycle bytes.

## Acceptance boundary

- A plan binds repository identity, synchronized `origin/main`, approved close,
  sequence-2 finalization, archive/evidence identities, and the six exact
  controlled documentation paths with before/after digests.
- Apply fails closed for stale, foreign, malformed, symlinked, dirty, partial,
  or unexpected state before any write. Re-applying a current plan is a
  deterministic no-op.
- WI-255’s English, Simplified Chinese, and Japanese projections become
  `Implemented` without changing its `.ai` archive, evidence, finalization, or
  close bytes.
- AGENTS and the three-language workflow/command references require
  `close → visible Outcome → post-close plan/apply → check-all → terminal CI`.
- Wrapper, promoter, documentation, manifest, governance, formatting, clippy,
  and locked workspace checks pass under the installed Runtime.

## Verification scenarios

The Contract covers valid plan/apply/idempotent rerun, typed identity and
staleness rejection, dirty/unexpected/partial projection rejection, and the
actionable terminal-CI handoff. The wrapper test uses isolated Git fixtures and
asserts that immutable `.ai` digests remain unchanged.

## References

- [Agent workflow](../reference/agent-workflow.md)
- [Commands](../reference/commands.md)
- [Reference parity](../reference/reference-parity.md)
