---
author: AI Cockpit maintainers
title: WI-677 — WI-657 lifecycle concurrency and recovery successor
description: Re-verify P2-C from the current default branch and serialize lifecycle commits across concurrent callers.
workItemId: WI-677-wi657-lifecycle-concurrency-successor
audience:
  - contributor
  - maintainer
  - reviewer
status: recovered
authority: human-authorized
lastVerifiedBy: WI-677-wi657-lifecycle-concurrency-successor
---

# WI-677 — WI-657 lifecycle concurrency and recovery successor

This successor re-delivers the P2-C boundary from the current `origin/main`.
The archived WI-657 branch fixed a same-process temporary-file collision but
was based on an older default branch and left a documented rollback-clobber
gap. Its evidence remains historical; WI-683 owns the redelivery from the latest default base.

## Boundary

`finish`, `archive`, `close`, recovery-decision recording, and active-artifact
reconciliation now share a per-Work-Item operating-system file lock under the
ignored `.ai/locks/` runtime directory. The lock is retained as a stable inode
and released by the OS when the handle or process exits. This serializes both
threads and independent CLI processes without adding a database, global
repository cache, or new crate. `atomic_write` also combines the process id
with the existing atomic sequence counter, so same-process writers never
reuse one temporary pathname.

The lock covers failure projection persistence for `finish` as well as the
main transition. A losing caller therefore observes the committed state and
returns a business-level rejection; it cannot restore a stale pre-attempt
snapshot over a successful sibling. Existing JSON schemas, lifecycle names,
authorization checks, and archive layout remain unchanged.

## Controlled verification

`lifecycle_concurrency.rs` covers:

- two threads finishing the same Work Item;
- two independent test processes finishing the same Work Item;
- concurrent archive and close calls, with exactly one commit for each;
- missing active projections and corrupt archive projections failing closed
  before a new commit is recorded.

The final summary, outcome, archive manifest, and close decision are parsed as
JSON after the concurrent runs. Raw filesystem race errors are rejected. Any
future crash-consistency limitation must remain an explicit recovery or
unknown state; this Work Item does not infer authorization from a projection.

## Verification and remaining risk

The required Rust, formatting, Clippy, documentation, and governance checks
run on the exact successor head. The lock protects processes that use this
repository implementation; external writers that bypass it remain outside
the repository API boundary and are not claimed safe.
