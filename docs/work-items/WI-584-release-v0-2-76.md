---
author: AI Cockpit maintainers
title: "WI-584 — v0.2.76 release and object-adopter recovery handoff"
description: "Publish and validate the Runtime release required by archived Work Item revalidation."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-584-release-v0-2-76
lastVerifiedBy: WI-584-release-v0-2-76
---

[简体中文](WI-584-release-v0-2-76.zh-CN.md) · [日本語](WI-584-release-v0-2-76.ja.md)

# WI-584 — v0.2.76 release and object-adopter recovery handoff

## Objective

Publish v0.2.76 from the reviewed, synchronized default branch as an
identity-bound Runtime baseline. The release is the Runtime dependency for the
object repository's append-only archived-Contract revalidation and successor
close acceptance; this Work Item does not operate on that repository.

## Scope and boundary

- Align Cargo metadata, lockfile, and tri-lingual release/versioning guidance
  for v0.2.76, retaining v0.2.75 as the preceding public baseline.
- Produce and validate the identity-bound release archives, manifest,
  checksums, SBOM, provenance, attestation, and Runtime identity.
- Run public adopter and N-1 acceptance from downloaded immutable artifacts in
  isolated roots, including forbidden-root and temporary-run cleanup proofs.
- Document the object-adopter handoff as an external read-only acceptance
  dependency; never edit its `.ai/`, source, branches, or evidence here.

Runtime behavior, the object repository, global Agent/MCP configuration,
reference-source copying, failed-tag history, and unrelated historical records
are outside this Work Item.

## Acceptance

1. Cargo metadata, lockfile, and release/versioning pages identify v0.2.76 and
   retain v0.2.75 as the preceding public baseline.
2. Release CI produces identity-bound five-target artifacts and supply-chain
   receipts for v0.2.76.
3. Public adopter and N-1 acceptance use only downloaded v0.2.76 artifacts,
   prove forbidden-root isolation and temporary-run cleanup, and exercise the
   same binary against this repository.
4. No Runtime behavior, object repository, global configuration, failed-tag
   history, or unrelated evidence is changed.
5. A published Runtime identity and exact command handoff are recorded for the
   object repository team; no historical evidence is rewritten or fabricated.

## Verification boundary

Contract acceptance remains authoritative in its authoring language; localized
pages change presentation only. Object-repository recovery is an external
read-only handoff and is not claimed complete by this Work Item.

## Verification

- `tests/release/version_consistency.sh`
- `tests/release/workflow_policy.sh`
- `tests/release/action_runtime_policy.sh`
- `tests/release/source_archive_policy_test.sh`
- `tests/release/adopter_acceptance_test.sh`
- `tests/release/adopter_upgrade_acceptance_test.sh`
- `tests/docs/documentation_acceptance.sh`
- `tests/docs/parity_status_check.sh`
- `cargo test --locked --workspace`
- `git diff --check`
