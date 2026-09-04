---
author: AI Cockpit maintainers
title: "WI-574 — v0.2.75 release and public-artifact acceptance"
description: "Publish and validate the next immutable AI Cockpit Runtime release."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-574-release-v0-2-75
lastVerifiedBy: WI-574-release-v0-2-75
terminalArchive: .ai/work-items/archive/WI-574-release-v0-2-75.contract.json
terminalVerification: .ai/evidence/WI-574-release-v0-2-75.verification.json
terminalFinalization: .ai/decisions/WI-574-release-v0-2-75.finalize.json
terminalDecision: .ai/decisions/WI-574-release-v0-2-75.close.json
---

[简体中文](WI-574-release-v0-2-75.zh-CN.md) · [日本語](WI-574-release-v0-2-75.ja.md)

# WI-574 — v0.2.75 release and public-artifact acceptance

## Objective

Publish v0.2.75 from the reviewed, synchronized default branch as an
immutable Runtime baseline, then prove that its downloaded public binary can
govern this repository without a source checkout or workspace fallback.

## Scope and boundary

- Align Cargo metadata, lockfile, and release/versioning guidance in all three
  languages for v0.2.75.
- Bind the release to the closed reference-comparison and documentation
  promotion records already present on the default branch.
- Produce and validate the identity-bound five-target archives, manifest,
  checksums, SBOM, provenance, attestation, and Runtime identity.
- Run public adopter and N-1 acceptance from immutable downloaded artifacts in
  isolated roots, including forbidden-root and temporary-run cleanup proofs.

The Runtime implementation, object repository, global Agent/MCP configuration,
reference-source implementation copying, failed-tag rewriting, and unrelated
historical records are outside this Work Item.

## Acceptance

1. Cargo metadata, lockfile, and current release/versioning pages identify
   v0.2.75 while retaining v0.2.74 as the preceding public baseline.
2. Release CI produces the identity-bound five-target artifact and
   supply-chain receipt set for v0.2.75.
3. Public adopter and N-1 acceptance use only v0.2.75 downloaded artifacts,
   prove isolation and cleanup, and exercise the same binary against this
   repository.
4. The release starts from a synchronized ready default branch and changes no
   Runtime behavior, object repository, global configuration, or unrelated
   historical evidence.
5. Any release or adopter failure preserves immutable release truth, records the
   failure receipt, and never falls back to a source checkout or workspace binary.

## Verification boundary

Contract acceptance remains authoritative in its authoring language; localized
pages change presentation only. A public Release is not accepted until its
immutable assets and adopter receipts are verified. Object-repository
acceptance is an external read-only handoff and is not claimed here.

## Verification

- `tests/release/version_consistency.sh`
- `tests/release/workflow_policy.sh`
- `tests/release/action_runtime_policy.sh`
- `tests/release/source_archive_policy_test.sh`
- `tests/release/adopter_acceptance_test.sh`
- `tests/release/adopter_upgrade_acceptance_test.sh`
- `tests/docs/documentation_acceptance.sh`
- `tests/docs/parity_status_check.sh`
- `git diff --check`
