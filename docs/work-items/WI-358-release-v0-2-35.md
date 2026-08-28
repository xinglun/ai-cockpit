---
author: AI Cockpit maintainers
title: "WI-358 — v0.2.35 release and lifecycle-entry compatibility"
workItemId: WI-358-release-v0-2-35
description: "Publish the adopter cleanup-order fix and prevent legacy close records from deadlocking new Work Items."
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-358-release-v0-2-35
terminalArchive: .ai/work-items/archive/WI-358-release-v0-2-35.contract.json
terminalVerification: .ai/evidence/WI-358-release-v0-2-35.verification.json
terminalFinalization: .ai/decisions/WI-358-release-v0-2-35.finalize.json
terminalDecision: .ai/decisions/WI-358-release-v0-2-35.close.json
capabilityClaims: [release_distribution, lifecycle_entry_compatibility]
---

# WI-358 — v0.2.35 release and lifecycle-entry compatibility

[简体中文](WI-358-release-v0-2-35.zh-CN.md) · [日本語](WI-358-release-v0-2-35.ja.md)

## Intent

Publish the merged adopter acceptance ordering fix as a public v0.2.35
Release. Preserve the fail-closed close gate for newly archived Work Items,
while treating pre-marker archive bytes as historical so they cannot block a
new Work Item indefinitely.

## Scope

- Add an explicit `closeRequired` marker to new archive manifests.
- Keep current archives without that marker historical at the new-entry gate;
  invalid or marked-current close records remain blocking.
- Add repository regression coverage for both historical and current archives.
- Align Cargo versions and current tri-language release/versioning documents;
  retain the failed v0.2.34 publication fact.
- Publish only through the reviewed hosted release workflow and validate the
  exact public artifact and adopter acceptance after publication.

## Boundary

This Work Item does not rewrite historical Contract, close, evidence, or
archive bytes; it does not infer a human decision or mutate a Homebrew tap.
Post-release failure remains `releasePublished: true` with acceptance failure.

## Acceptance

1. Workspace packages and `Cargo.lock` resolve to 0.2.35 and the tag is
   `v0.2.35`.
2. New archive manifests carry `closeRequired: true`; marked-current archives
   without a valid identity-bound close remain blocked.
3. Pre-marker historical archives do not deadlock new Work Item entry and are
   not promoted to a current green Outcome.
4. Source documentation, release policy, version consistency, and workspace
   verification pass before tagging.
5. Hosted Release artifacts bind manifest, `SHA256SUMS`, SBOM, provenance, and
   staged adopter checks; public acceptance proves downloaded-binary identity,
   lifecycle, isolation, evidence reuse, and temporary-root cleanup.

## Verification

The Runtime lifecycle evidence, hosted PR checks, release workflow run, public
binary digest, and adopter acceptance receipt are the authoritative records.
Terminal lifecycle: archive `.ai/work-items/archive/WI-358-release-v0-2-35.contract.json`;
verification `.ai/evidence/WI-358-release-v0-2-35.verification.json`; finalization
`.ai/decisions/WI-358-release-v0-2-35.finalize.json`; close
`.ai/decisions/WI-358-release-v0-2-35.close.json`.
