---
author: AI Cockpit maintainers
title: "WI-213 — v0.2.27 immutable release and adopter acceptance"
description: "Publish v0.2.27 from merged main and validate the public artifact with the installed Runtime."
audience:
  - maintainer
  - adopter
  - reviewer
workItemId: WI-213-release-v0-2-27
status: current
authority: canonical
lastVerifiedBy: WI-213-release-v0-2-27
---

# WI-213 — v0.2.27 immutable release and adopter acceptance

This Work Item publishes the first release after the v0.2.26 source-quality
failure. The v0.2.26 tag remains immutable failed history; it is not rewritten
or reused. v0.2.27 is built from merged PR #160 and is accepted only through
the public downloaded artifact.

## Acceptance

1. Cargo version, lockfile, release documentation, tri-language parity, and
   release workflow policy consistently identify v0.2.27.
2. The tag is created only on the merged PR #160 descendant and is protected by
   the release workflow's source, tag, manifest, checksum, and provenance gates.
3. Public adopter and N-1 acceptance use downloaded immutable artifacts only;
   source checkout, workspace binaries, and local `target` files are forbidden.
4. Acceptance receipts bind repository/runtime identity, isolation manifests,
   cleanup state, evidence reuse, and the visible localized Outcome.
5. After publication, the installed v0.2.27 Runtime completes WI-212's
   post-merge finalization transition, `finalize-verify`, and structured close.

## Out of scope

Reference-source file-by-file parity is the next batch. This Work Item does not
add unrelated Runtime features or modify user-global Agent/MCP configuration.

## Evidence boundary

The published Release, downloaded archive, manifest, checksums, attestation,
and adopter receipts are immutable external evidence. A post-release failure
records `releasePublished: true` and `adopterAcceptance: failed`; it never
rewrites Release truth.
