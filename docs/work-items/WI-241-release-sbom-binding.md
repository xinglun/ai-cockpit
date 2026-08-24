---
author: AI Cockpit maintainers
title: "WI-241 — Release SBOM artifact binding"
workItemId: WI-241-release-sbom-binding
description: "Bind each future target SBOM to exact packaged bytes and close the public release asset inventory."
audience:
  - maintainer
  - reviewer
  - adopter
status: implemented
lastVerifiedBy: WI-241-release-sbom-binding
authority: canonical
---

# WI-241 — Release SBOM artifact binding

WI-241 corrects the release-construction boundary found during the v0.2.31
enterprise-compliance audit. It changes later candidates only; the public
v0.2.31 tag, Release assets, checksums, attestations, and acceptance receipts
remain immutable historical truth.

## Delivered boundary

- `cockpit-release bind-sbom` calculates the SHA-256 of the actual staged
  archive and the executable member extracted from that archive. It inserts a
  standard SPDX 2.3 release Package and File linked by `DESCRIBES` and
  `CONTAINS`, then validates the resulting document before writing it.
- The validator requires the exact target, canonical version, target-named
  archive/SBOM filenames, one reserved Package, one reserved File, one of each
  binding relationship, and matching nonzero SHA-256 values.
- The Anchore dependency scan is retained, but its automatic artifact and
  Release uploads are disabled. Only the five target-named SBOMs can enter the
  candidate, attestation, and publication allowlists.
- Checksums are generated after the Formula. They cover the five archives,
  five SBOMs, canonical manifest, and Formula exactly once in stable order.
  The checksum file itself is the thirteenth published asset and cannot
  checksum itself.
- Candidate validation rejects missing or orphan publishable assets, duplicate
  checksum names, unsorted or malformed lines, missing entries, extra entries,
  and digest mismatches before downstream staged adopter acceptance.

## Evidence boundary

An SPDX filename or dependency scan is not adopter acceptance. The SBOM proves
only its exact archive/binary binding. Hosted attestation and the existing
staged/public adopter acceptance jobs remain separate downstream gates.

Regression coverage is in `crates/cockpit-release/tests/sbom.rs`,
`crates/cockpit-release/tests/manifest.rs`, and
`tests/release/workflow_policy.sh`. Runtime verification is recorded at
`.ai/evidence/WI-241-release-sbom-binding.verification.json`.

## References

- [Release distribution](../release/distribution.md)
- [Reference source parity](../reference/reference-parity.md)
