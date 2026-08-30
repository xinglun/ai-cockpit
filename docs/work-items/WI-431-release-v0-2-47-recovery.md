---
author: AI Cockpit maintainers
title: "WI-431 — v0.2.47 release recovery"
description: Recover the failed v0.2.46 publication without moving its immutable tag.
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
workItemId: WI-431-release-v0-2-47-recovery
lastVerifiedBy: WI-431-release-v0-2-47-recovery
terminalArchive: .ai/work-items/archive/WI-431-release-v0-2-47-recovery.contract.json
terminalVerification: .ai/evidence/WI-431-release-v0-2-47-recovery.verification.json
terminalFinalization: .ai/decisions/WI-431-release-v0-2-47-recovery.finalize.json
terminalDecision: .ai/decisions/WI-431-release-v0-2-47-recovery.close.json
---

# WI-431 — v0.2.47 release recovery

## Intent and boundary

The first v0.2.46 publication attempt was rejected by the release source gate
because closed Work Item documentation had not yet been promoted. The tag is
kept as immutable failed-delivery history; it is never moved or relabeled.
This Work Item promotes that terminal documentation before creating a new
patch release and proves the public artifact path end to end.

This is a release/documentation recovery. It does not change Runtime source,
CI workflow policy, or the Repository Protocol.

## Acceptance

- Closed Work Item documentation is promoted in all three languages before
  release tagging.
- Cargo metadata and lockfile advance exactly one patch from v0.2.46 to
  v0.2.47; the failed v0.2.46 tag is not reused.
- Public v0.2.47 artifacts pass manifest, checksum, SBOM, provenance,
  platform smoke, adopter, and N-1 acceptance using downloaded artifacts only.
- The v0.2.46 failure remains documented as an unpublished immutable tag.
- Reviewed merge, finalization, close, synchronization, and exact cleanup
  leave the default branch `ready_on_base`.

## Verification boundary

The release route runs the repository's strict gate manifest, the workspace
tests, the three-language documentation checks, and the public release
harness. The post-release receipt must bind the downloaded binary, release
manifest, checksums, runtime identity, adopter repository identity, isolation
manifests, and cleanup result.
