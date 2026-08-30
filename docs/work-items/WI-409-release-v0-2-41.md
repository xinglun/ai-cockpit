---
author: AI Cockpit maintainers
title: "WI-409 — v0.2.41 release and adopter acceptance"
description: "Publish the reviewed post-WI-408 Runtime and verify the immutable artifact in a fresh adopter."
workItemId: WI-409-release-v0-2-41
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-409-release-v0-2-41
capabilityClaims: [release_distribution, adopter_acceptance, repository_isolation]
---

# WI-409 — v0.2.41 release and adopter acceptance

[简体中文](WI-409-release-v0-2-41.zh-CN.md) · [日本語](WI-409-release-v0-2-41.ja.md)

## Intent

Publish v0.2.41 from the reviewed post-WI-408 `main` and prove that the
downloaded immutable Release binary can attach and govern a fresh adopter
without copying reference-source or V1 runtime residue.

## Boundary

This Work Item advances the patch version, updates current tri-language release
and versioning guidance, runs the strict release workflow, and records public
adopter/N-1 acceptance. It does not change governance semantics, historical
evidence, global Agent/MCP configuration, or unrelated adopter application
source. Runtime-only distribution remains separate from repository attachment.

## Acceptance

1. Cargo metadata and lockfile advance exactly one patch from v0.2.40 to
   v0.2.41, without reusing an existing tag or Release.
2. The reviewed workflow builds the declared targets from the exact merged
   commit and binds the manifest, Formula, SHA256SUMS, SBOM, provenance, and
   immutable tag/Release identity.
3. The downloaded public v0.2.41 binary is checksum-verified and records
   runtime version, archive digest, binary digest, platform, and download
   source without source or workspace fallback.
4. Fresh adopter acceptance proves attach/profile/agent doctor, the
   `first-adopter-smoke` `not_ready` boundary, lifecycle and evidence reuse,
   repository/runtime isolation, and temporary-root cleanup.
5. The current repository and a fresh adopter inherit WI-408's read-only
   `work-item inspect` boundary through the shared Runtime.
6. Reviewed merge, finalization, close, synchronization, exact branch cleanup,
   and release documentation promotion leave `main` ready on base.

## Verification boundary

Pre-release checks use the strict source and staged release gates. Post-release
checks download only immutable public artifacts and persist runtime, checksum,
adopter, isolation, and cleanup evidence. A release failure never rewrites
published Release truth. ORG-X and other adopter repositories are inspected
without copying reference-source residue.
