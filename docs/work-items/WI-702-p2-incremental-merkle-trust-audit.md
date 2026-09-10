---
author: AI Cockpit maintainers
title: "WI-702 — P2 IncrementalMerkle trust audit"
description: "Validate metadata reuse boundaries before any incremental content identity computation can affect governance."
audience: [maintainer, reviewer, adopter]
workItemId: WI-702-p2-incremental-merkle-trust-audit
status: recovered
authority: human:repository-owner
lastVerifiedBy: WI-712-wi702-finalization-recovery
recoveryDecision: .ai/decisions/WI-702-p2-incremental-merkle-trust-audit.recovery.json
---

[简体中文](WI-702-p2-incremental-merkle-trust-audit.zh-CN.md) · [日本語](WI-702-p2-incremental-merkle-trust-audit.ja.md)

# WI-702 — P2 IncrementalMerkle trust audit

## Intent and boundary

This Work Item validates the trust boundary of `IncrementalMerkle` before any
future incremental computation is considered. The North Star remains
Calibrated Human-Agent Trust: repository isolation, Work Item isolation,
authorization boundaries, evidence binding, and recovery remain more important
than a cache-hit metric.

The current repository call-graph search finds the `IncrementalMerkle` type and
its `refresh` calls only in `crates/cockpit-git` itself and its snapshot tests;
no production governance, MCP, doctor, status, or outcome path constructs this
helper. Therefore this Work Item does not claim a protected-decision effect or
a production performance gain. It changes only the helper and its focused
tests; the other agents' trees, branches, pull requests, and evidence remain
outside the boundary.

## Hypothesis and observed defect

The pre-change helper skipped a read when the cached size and mtime matched.
Those metadata values are clues, not proof of content identity: a same-length
replacement with restored mtime could reuse a stale digest. That would be
unacceptable if the helper were later connected to a protected decision.

The candidate therefore rereads and hashes every declared regular file on each
refresh. It checks metadata before and after the read and returns the explicit
`ChangedDuringRead` error when a detectable size, timestamp, disappearance, or
file-type change occurs during the read. The public `files_reused` field is
retained for compatibility and is now always zero; no caller may interpret
metadata as proof of an unchanged file.

## Correctness evidence

The focused `cockpit-git` suite covers:

- same-length content changed with the original mtime restored;
- replacement, deletion, rename, and file-to-directory transitions;
- path escape rejection;
- a deterministic before/after metadata change during a read, which fails with
  `ChangedDuringRead`.

At the base revision, the old test explicitly expected unchanged files to be
reused. The candidate test now proves that unchanged files are reread and that
the same-length/restored-mtime case changes the Merkle root. This is a trust
correction, not an optimization; the candidate may cost more if a future
production caller uses it.

## Limits and governance state

The metadata guard cannot prove a concurrent edit that changes bytes and then
restores every observed metadata value while the read is in flight. That case
remains an explicit validity limitation, not a cache hit or a correctness
claim. Filesystem notifications, persistent indexes, a layered Merkle tree,
production integration, release behavior, and performance benchmarking are
out of scope.

This predecessor report intentionally does not claim a standalone green
terminal outcome. WI-712 completed the append-only recovery, reviewed PR
reconciliation, finalization, close, and documentation promotion; those
records are linked in the recovery boundary below. The predecessor bytes remain
immutable, and this Work Item still makes no production performance claim.

## Post-merge recovery boundary

PR #700 was reviewed and merged at `bd00a7ce888c2d0dba012da21ba1616eeeab0014`
with reviewed head `4eedf23ddf1e4a0491fb978127d61d852e6a5a1f`. The immutable
pre-merge finalization root for this Work Item binds `86f8535f`; the intervening
range also modifies the pending parity registry, so the installed Runtime
correctly rejects treating the range as an append-only finalization transition.

WI-702's archive, verification, Outcome, Events, Contract, and finalization
bytes remain historical evidence. The append-only recovery decision is
`.ai/decisions/WI-702-p2-incremental-merkle-trust-audit.recovery.json`, and
WI-712 owns the fresh post-merge finalization, parity registration, and exact
cleanup boundary. This recovery preserves the Calibrated Human-Agent Trust
North Star and makes no performance-benefit claim.
