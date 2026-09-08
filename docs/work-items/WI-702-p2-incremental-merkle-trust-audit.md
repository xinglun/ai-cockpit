---
author: AI Cockpit maintainers
title: "WI-702 — P2 IncrementalMerkle trust audit"
description: "Validate metadata reuse boundaries before any incremental content identity computation can affect governance."
audience: [maintainer, reviewer, adopter]
workItemId: WI-702-p2-incremental-merkle-trust-audit
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-702-p2-incremental-merkle-trust-audit
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

This report does not claim a final PR, merge, or green governance outcome. The
Runtime verification receipt, hosted review, archive, close, and final
documentation promotion remain required.
