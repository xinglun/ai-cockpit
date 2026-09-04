---
author: AI Cockpit maintainers
title: "WI-565 — v0.2.73 release and public-artifact acceptance"
description: "Publish and validate the next immutable AI Cockpit Runtime release."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-565-release-v0-2-73
lastVerifiedBy: WI-565-release-v0-2-73
terminalArchive: .ai/work-items/archive/WI-565-release-v0-2-73.contract.json
terminalVerification: .ai/evidence/WI-565-release-v0-2-73.verification.json
terminalFinalization: .ai/decisions/WI-565-release-v0-2-73.finalize.json
terminalDecision: .ai/decisions/WI-565-release-v0-2-73.close.json
---

[简体中文](WI-565-release-v0-2-73.zh-CN.md) · [日本語](WI-565-release-v0-2-73.ja.md)

# WI-565 — v0.2.73 release and public-artifact acceptance

## Objective

Publish v0.2.73 from the reviewed default branch as an immutable Runtime
baseline, then prove that its downloaded public binary can govern this
repository without using a source checkout or workspace fallback.

## Scope and boundary

- Align Cargo metadata, lockfile, and current release/versioning guidance in
  English, Simplified Chinese, and Japanese.
- Bind the release to the reviewed reference-comparison and documentation
  promotion records already closed on the default branch.
- Produce and validate the five-target archives, manifest, checksums, SBOM,
  provenance, attestation, and Runtime identity.
- Run public adopter and N-1 acceptance from immutable downloaded artifacts in
  isolated roots, including forbidden-root and temporary-run cleanup proofs.

The object repository, global Agent/MCP configuration, Runtime behavior,
source-template copying, failed-tag rewriting, and unrelated historical
records are outside this Work Item.

## Acceptance

1. Cargo metadata, lockfile, and current release/versioning pages identify
   v0.2.73; v0.2.72 remains the immediately preceding public baseline.
2. Release CI produces the identity-bound five-target artifact and
   supply-chain receipt set.
3. Public adopter and N-1 acceptance use only v0.2.73 downloaded artifacts,
   prove isolation and cleanup, and exercise the same binary against this
   repository.
4. The release starts from a synchronized ready default branch and changes no
   Runtime behavior, object repository, global configuration, or unrelated
   historical evidence.

## Verification boundary

Contract acceptance remains authoritative in its authoring language; localized
pages change presentation only. A public Release is not considered accepted
until its immutable assets and adopter receipts are verified. Object-repository
acceptance is an external read-only handoff and is not claimed by this page.
