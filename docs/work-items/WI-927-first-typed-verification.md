---
author: AI Cockpit maintainers
workItemId: WI-927-first-typed-verification
title: First typed required verification execution boundary
description: Allow the first declared required verification to create its formal receipt without weakening terminal lifecycle gates.
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: authorized
lastVerifiedBy: WI-927-first-typed-verification
---

# WI-927 — First typed verification execution boundary

This Work Item continues the Issue #893 repair after the predecessor Contract
was retired as malformed. It repairs the Runtime-only precondition loop where a
valid checkpointed Work Item cannot execute its first declared typed required
verification because the Summary entries are created only by that verification.

The Runtime may execute declared checks once when the Contract, repository
snapshot, checkpoint, and non-red preflight are current and the only pending
governance facts are missing typed verification entries. A formal identity-bound
receipt must still populate the Summary before `finish`, `archive`, or `close`.
Failed, duplicate, stale, foreign, and malformed evidence remains fail-closed
before process spawn.

The user-visible benefit is not declared until the focused and hosted evidence
is complete.
