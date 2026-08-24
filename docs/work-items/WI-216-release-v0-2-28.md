---
author: AI Cockpit maintainers
title: "WI-216 — v0.2.28 immutable release and adopter acceptance"
description: "Publish v0.2.28 from the merged reference comparison baseline and validate the public artifact with the installed Runtime."
audience:
  - maintainer
  - adopter
  - reviewer
workItemId: WI-216-release-v0-2-28
status: recovered
authority: canonical
lastVerifiedBy: WI-216-release-v0-2-28
---

# WI-216 — v0.2.28 immutable release and adopter acceptance

This Work Item publishes the patch Release after the first reference-source
file-comparison batch. The comparison baseline is already merged; this boundary
only establishes the next immutable Runtime artifact and its public acceptance.

## Acceptance

1. Workspace package versions, lockfile, release documentation, architecture
   documentation, and tri-language routes consistently identify v0.2.28.
2. The tag is created only from the reviewed merged default-branch descendant
   and is protected by source, tag, manifest, checksum, and provenance gates.
3. Public adopter and N-1 acceptance use downloaded immutable artifacts only;
   source checkout, workspace binaries, and local `target` files are forbidden.
4. Acceptance receipts bind repository/runtime identity, isolation manifests,
   cleanup state, evidence reuse, and the visible localized Outcome.
5. Post-release version consistency, adopter acceptance, and upgrade acceptance
   pass without rewriting Release truth.

## Out of scope

The next reference-source file-comparison batch is separate. This Work Item
does not add Runtime features, copy reference implementation code, or modify
user-global Agent/MCP configuration.

## Evidence boundary

The published Release, downloaded archive, manifest, checksums, attestation,
and adopter receipts are immutable external evidence. A post-release failure
records `releasePublished: true` and `adopterAcceptance: failed`; it never
rewrites Release truth.
