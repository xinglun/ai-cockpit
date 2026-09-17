---
author: AI Cockpit maintainers
workItemId: WI-890-release-v0-2-95
title: Governed v0.2.95 release after four-direction convergence
description: Publish v0.2.95 only after the Outcome, HCI, four-direction, Issue #851, interface-discovery, Rust/toolchain, and current performance evidence are verified.
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-890-release-v0-2-95
---

# WI-890 — Governed v0.2.95 release after four-direction convergence

This Work Item is the final publication route requested after the Outcome
delivery repair, the four-direction convergence work, Issue #851, the BAML-
inspired interface-discovery trial, and the Rust/toolchain update have been
completed. Publication is not complete until the reviewed candidate is merged,
the provider Release and immutable tag exist, and a downloaded artifact passes
the release acceptance harness.

## Acceptance boundary

- Preserve v0.2.93 as the N-1 baseline and publish only from the reviewed,
  synchronized main commit for v0.2.95.
- Verify the release package, checksums, manifests, provider Release assets,
  and downloaded fresh-install/N-1 upgrade acceptance in an isolated root.
- Keep the documented unknowns explicit: current performance evidence shows no
  regression beyond noise but does not prove development-cycle improvement;
  default host display confirmation remains unknown without a configured host.
- Do not change object-repository main branches, add product scope, or claim
  that a generated report proves user-visible conversation display.

## Verification plan

Run the declared format, governance, documentation, package, candidate, and
release acceptance checks before provider mutation. Use the shared verification
target with `CARGO_INCREMENTAL=0`; record commands, exit codes, logs, artifact
digests, environment identity, and cleanup evidence. A failed acceptance must
preserve its evidence and must not be repaired by republishing the same tag.

## Publication and post-release evidence

After reviewed merge and green hosted checks, publish the immutable v0.2.95
tag/Release through the repository's release workflow. Download the public
assets with `gh release download` and run the acceptance harness against those
downloads, not a workspace build. Record installation/upgrade status, release
links, checksum and manifest identity, and exact temporary-root cleanup before
finalization and close.
