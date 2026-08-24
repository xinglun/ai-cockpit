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

This Work Item establishes the pre-release baseline for the next immutable
public Runtime release. It updates package identity and reader-facing release
documentation, then binds the reviewed PR and pre-merge finalization boundary.
Public artifact identity, installed-binary checks, adopter lifecycle, and N-1
upgrade are post-release facts and are handed to a successor Work Item after
the reviewed merge; this Work Item does not claim them early.

## Acceptance boundary

- Workspace metadata and `Cargo.lock` identify v0.2.30 consistently.
- Release, versioning, distribution, and English/Chinese/Japanese parity
  documents identify v0.2.30, with v0.2.29 as the immediate N-1 baseline.
- Source quality, release policy, version consistency, and documentation gates
  pass before publication.
- The reviewed PR has a valid pre-merge finalization boundary; the public Release
  tag is created only after merge. No source checkout or workspace binary is
  accepted as public-release evidence.
- A successor Work Item verifies the installed v0.2.30 binary and isolated
  adopter/upgrade harnesses after publication. Temporary run roots are cleaned
  while successor acceptance receipts remain auditable.

## References

- [Release and Distribution](../release/distribution.md)
- [Versioning](../architecture/versioning.md)
- [Reference parity ledger](../reference/reference-parity.md)
- [Public adopter acceptance harness](../../tests/release/adopter_acceptance.sh)
