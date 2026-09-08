---
author: AI Cockpit maintainers
title: WI-683 — P2-C lifecycle concurrency and recovery redelivery
description: Redeliver the lifecycle concurrency and recovery boundary from the latest default branch under a unique Work Item identity.
workItemId: WI-683-p2c-lifecycle-concurrency-recovery
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-683-p2c-lifecycle-concurrency-recovery
terminalArchive: .ai/work-items/archive/WI-683-p2c-lifecycle-concurrency-recovery.contract.json
terminalVerification: .ai/evidence/WI-683-p2c-lifecycle-concurrency-recovery.verification.json
terminalFinalization: .ai/decisions/WI-683-p2c-lifecycle-concurrency-recovery.finalize.json
terminalDecision: .ai/decisions/WI-683-p2c-lifecycle-concurrency-recovery.close.json
---

# WI-683 — P2-C lifecycle concurrency and recovery redelivery

This Work Item redelivers the P2-C boundary from the current `origin/main`
after the earlier redelivery identity collided with an unrelated WI-681. The
WI-677 archive, evidence, and recovery decision remain immutable historical
records; this Work Item owns the fresh implementation and verification.

## Boundary

`finish`, `archive`, `close`, recovery-decision recording, and active-artifact
reconciliation share a per-Work-Item operating-system file lock under the
ignored `.ai/locks/` runtime directory. The lock is retained as a stable inode
and released by the OS when the handle or process exits. This serializes both
threads and independent CLI processes without adding a database, global
repository cache, or new crate. `atomic_write` combines the process id with the
existing atomic sequence counter so same-process writers never reuse one
temporary pathname.

The lock covers failure projection persistence for `finish` as well as the main
transition. A losing caller therefore observes the committed state and returns
a business-level rejection; it cannot restore a stale pre-attempt snapshot over
a successful sibling. Existing JSON schemas, lifecycle names, authorization
checks, archive layout, and predecessor bytes remain unchanged.

## Controlled verification

`lifecycle_concurrency.rs` covers:

- two threads finishing the same Work Item;
- two independent test processes finishing the same Work Item;
- concurrent archive and close calls, with exactly one commit for each;
- missing active projections and corrupt archive projections failing closed
  before a new commit is recorded.

The final summary, outcome, archive manifest, and close decision are parsed as
JSON after concurrent runs. Raw filesystem race errors are rejected. Any crash
consistency limitation remains an explicit recovery or unknown state; this
Work Item does not infer authorization from a projection.

## Verification and remaining risk

The required Rust, formatting, Clippy, documentation, governance, and hosted
checks run on the exact successor head. The lock protects processes using this
repository implementation; external writers that bypass it remain outside the
repository API boundary and are not claimed safe.
