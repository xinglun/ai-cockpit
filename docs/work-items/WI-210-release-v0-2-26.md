---
author: AI Cockpit maintainers
title: "WI-210 — v0.2.26 immutable release and adopter acceptance"
description: "Publish v0.2.26 from the merged default branch and close the release governance transition with the public binary."
audience:
  - maintainer
  - adopter
  - reviewer
workItemId: WI-210-release-v0-2-26
status: current
authority: canonical
lastVerifiedBy: WI-210-release-v0-2-26
---

# WI-210 — v0.2.26 immutable release and adopter acceptance

This Work Item establishes the next immutable public release after the
failed, preserved v0.2.25 publication history. It binds version consistency,
the merged PR and release-tag proof, public binary adopter/upgrade acceptance,
and the installed Runtime finalization and structured close for WI-209.

The adopter boundary uses only downloaded public Release assets. Source
checkout, `cargo build`, `cargo run`, workspace binaries, and local `target`
artifacts are not acceptable fallbacks. `v0.2.25` remains immutable failed
history and is never moved or reused.

## Acceptance

1. v0.2.26 version, distribution documentation, and tri-language parity are
   consistent before release verification.
2. The immutable tag is created only on a merged PR commit with a valid
   premerge-finalize receipt and release-tag ancestor proof.
3. Public adopter and N-1 upgrade acceptance pass from downloaded artifacts,
   with repository/runtime identity and isolation evidence.
4. Temporary acceptance roots are cleaned on success, failure, and
   interruption; cleanup is included in the checksummed receipt.
5. The installed public Runtime completes WI-209 finalize,
   finalize-verify, and structured human close, with a visible localized
   Outcome handoff.

## Out of scope

Reference-source file-by-file parity expansion is the next batch. This Work
Item does not add unrelated Runtime features or modify user-global Agent/MCP
configuration.

## Evidence boundary

The published Release and its downloaded archive/manifests are immutable
external evidence. Post-release failure records `releasePublished: true` and
`adopterAcceptance: failed`; it never rewrites Release truth.
