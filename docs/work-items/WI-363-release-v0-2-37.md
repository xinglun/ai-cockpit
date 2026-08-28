---
author: AI Cockpit maintainers
title: "WI-363 — v0.2.37 release and installed-binary acceptance"
workItemId: WI-363-release-v0-2-37
description: "Publish the next immutable release after release-adopter cleanup and verify the public binary in an isolated adopter flow."
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-363-release-v0-2-37
capabilityClaims: [release_distribution, adopter_acceptance]
---

# WI-363 — v0.2.37 release and installed-binary acceptance

[简体中文](WI-363-release-v0-2-37.zh-CN.md) · [日本語](WI-363-release-v0-2-37.ja.md)

## Intent

Publish v0.2.37 from reviewed, synchronized `main` after the merged
release-adopter cleanup, then install and exercise only the immutable public
artifact against this repository.

## Scope and boundary

- Align workspace package metadata, lockfile, and current tri-language
  release/versioning documentation to v0.2.37.
- Use the reviewed hosted release workflow and immutable public artifacts,
  checksums, SBOM/provenance, adopter acceptance, and N-1 upgrade evidence.
- Install the checksum-verified public macOS ARM64 binary and run explicit
  repository health checks.
- Preserve the unpublished v0.2.36 staged-acceptance failure as history.

Runtime behavior changes, historical evidence rewrites, global Agent/MCP
configuration, source-build fallbacks, and a second technology-stack adopter
are outside this Work Item.

## Acceptance

1. Cargo metadata and lockfile report 0.2.37 consistently.
2. Reviewed CI and release policy checks pass before the immutable tag.
3. Public artifacts are downloaded from GitHub, checksum-bound, and never
   replaced by a source or workspace binary.
4. Public adopter and N-1 acceptance produce auditable receipts, prove runtime
   and repository identity, isolation, lifecycle evidence, and temporary-root
   cleanup.
5. The installed public binary passes `inspect`, `status`, `doctor`, and
   `agent doctor` with an explicit `--repo`.
6. Merge, finalization, close, and exact branch/worktree cleanup leave the
   repository ready on the synchronized default branch.

## Verification boundary

The Runtime lifecycle records the Contract, checkpoint, verification, archive,
finalization, and close evidence. Hosted workflow receipts and the post-release
adopter receipts are authoritative for public artifact claims. Historical
v0.2.36 failure bytes remain unchanged.
