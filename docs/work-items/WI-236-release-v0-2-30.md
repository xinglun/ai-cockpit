---
author: AI Cockpit maintainers
title: "WI-236 — v0.2.30 release baseline and public adopter acceptance"
workItemId: WI-236-release-v0-2-30
description: "Publish v0.2.30 from the merged default branch and verify the immutable public artifact with installed-runtime acceptance."
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-236-release-v0-2-30
---

# WI-236 — v0.2.30 release baseline and public adopter acceptance

This Work Item establishes the next immutable public Runtime release from the
merged default branch. It updates the package identity and reader-facing
release documentation, then binds the published archive, installed binary,
adopter lifecycle, N-1 upgrade, isolation manifests, and finalization receipts
to the same release identity.

## Acceptance boundary

- Workspace metadata and `Cargo.lock` identify v0.2.30 consistently.
- Release, versioning, distribution, and English/Chinese/Japanese parity
  documents identify v0.2.30, with v0.2.29 as the immediate N-1 baseline.
- Source quality, release policy, version consistency, and documentation gates
  pass before publication.
- The public Release tag and immutable artifacts are verified after publication;
  no source checkout or workspace binary is accepted as release evidence.
- The installed v0.2.30 binary passes inspect/status/doctor/agent doctor and the
  isolated adopter/upgrade harnesses. Temporary run roots are cleaned while
  acceptance receipts remain auditable.

## References

- [Release and Distribution](../release/distribution.md)
- [Versioning](../architecture/versioning.md)
- [Reference parity ledger](../reference/reference-parity.md)
- [Public adopter acceptance harness](../../tests/release/adopter_acceptance.sh)
