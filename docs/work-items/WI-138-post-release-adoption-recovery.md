---
workItemId: WI-138-post-release-adoption-recovery
status: complete
lastVerifiedBy: WI-138-close
author: AI Cockpit maintainers
title: "Post-release adopter acceptance and stale-state recovery"
description: "Public v0.2.11 adopter evidence and fail-closed stale-state recovery boundary."
audience:
  - maintainer
  - adopter
authority: canonical
---

# WI-138 — Post-release adopter acceptance and stale-state recovery

## Purpose

This Work Item records the first adopter acceptance performed with the public
`v0.2.11` Runtime and documents the safe recovery boundary discovered during
release preparation.

WI-137 was verified before the `v0.2.11` release commit was merged. Its
verification receipt is therefore bound to the earlier repository snapshot.
The Runtime correctly reports that receipt as stale/foreign after the merge;
this is not permission to edit the receipt or downgrade the check.

## Recovery rule

When a Work Item is already `finish_ready` and the repository changes before
archive, do not edit `.ai/work-items/**`, replace `repositorySnapshotDigest`,
or reuse the old verification receipt. Keep the historical bytes and create a
new explicitly authorized Work Item from the current repository snapshot. The
new Work Item must run the normal lifecycle with the current installed Runtime.
This preserves both the failed recovery boundary and the later valid evidence.

## Public acceptance evidence

- Release: [v0.2.11](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.11)
- Release workflow: [run 32578324451](https://github.com/xinglun/ai-cockpit/actions/runs/32578324451)
- Fresh adopter receipt: [artifact 9477249990](https://github.com/xinglun/ai-cockpit/actions/runs/32578324451/artifacts/9477249990)
- N-1 upgrade receipt: [artifact 9477256331](https://github.com/xinglun/ai-cockpit/actions/runs/32578324451/artifacts/9477256331)
- Repository-local acceptance summary: `.ai/evidence/WI-138-release-adopter-acceptance.json`

The public receipts record immutable release identity, repository IDs,
Runtime digests, `first-adopter-smoke = not_ready`, evidence reuse, the full
Work Item lifecycle, isolation manifests, and cleanup state.

## Acceptance boundary

The public fresh-adopter and N-1 jobs run on the Linux release targets. The
current macOS ARM installation was independently downloaded from the public
release, checksum-verified against `release-manifest.json`, and checked with
`inspect`, `status`, `doctor`, and `agent doctor` using an explicit `--repo`.

No source build, local workspace binary, historical evidence rewrite, or
global Agent/MCP configuration change is part of this acceptance.
