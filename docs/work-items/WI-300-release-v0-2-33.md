---
author: AI Cockpit maintainers
title: "WI-300 — v0.2.33 release and installation acceptance"
workItemId: WI-300-release-v0-2-33
description: "Publish the corrected runtime, verify immutable artifacts, and install the public binary for repository and adopter acceptance."
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-300-release-v0-2-33
authority: canonical
---

# WI-300 — v0.2.33 release preparation

## Intent

Prepare v0.2.33 from the reviewed default branch after WI-299 corrected the
adopter finalization base binding. The public artifact is verified and installed
only in the mandatory post-release successor WI-301.

## Scope

- Align workspace package versions and release examples on v0.2.33.
- Keep the failed staged v0.2.32 history explicit and immutable.
- Run source, documentation, policy, and full workspace verification before
  publication.
- Publish only through the hosted release workflow with manifest, checksums,
  SBOM, provenance, and artifact smoke evidence.
- Configure the hosted workflow and handoff for the mandatory post-release
  installation and adopter acceptance successor.

## Out of scope

This Work Item does not rewrite v0.2.32 history, add runtime governance
behavior, publish an external Homebrew tap, perform post-release installation
or adopter acceptance, expand the adopter technology matrix, or modify global
Agent/MCP configuration.

## Acceptance criteria

1. Every workspace package and Cargo.lock entry resolves to 0.2.33 and all
   three language documentation routes identify the same current baseline.
2. The failed staged v0.2.32 publication remains explicitly historical and has
   no public-Release claim.
3. Source version consistency, documentation acceptance, governance integrity,
   release policy, and complete workspace tests pass before tagging.
4. The hosted workflow publishes v0.2.33 with manifest, SHA256SUMS, target SBOM,
   provenance, and artifact smoke evidence bound to the tagged commit.
5. The reviewed release workflow is configured to publish only after the
   pre-publication gates and to hand off to WI-301 for public artifact checks.
6. WI-300 closes without claiming public artifact installation or adopter
   acceptance; those claims require immutable public Release evidence in WI-301.

## Verification

- `cargo test --locked --workspace`
- `bash tests/docs/documentation_acceptance.sh --repo <repo>`
- `bash tests/release/version_consistency.sh --repo <repo>`
- `bash tests/release/release_policy_test.sh`
- `bash tests/release/adopter_acceptance_test.sh`
- `bash tests/release/adopter_upgrade_acceptance_test.sh`
- Hosted release quality, Windows runtime, and behavioral-oracle checks.
- WI-301 post-release public manifest, checksum, installation, repository, and
  adopter acceptance receipts (not evidence for this pre-publication WI).

## Historical boundary

The v0.2.32 tag records a failed staged publication caused by the finalization
base-revision defect. Its failed truth is retained; this Work Item prepares the
new v0.2.33 release and does not repair history by rewriting it. WI-301 owns
all post-release public-artifact and adopter claims.
