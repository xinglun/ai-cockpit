---
author: AI Cockpit maintainers
title: "WI-397 — v0.2.40 release and published performance inheritance"
description: "Publish the WI-396 clean-snapshot optimization and verify the downloaded release binary in this repository and a fresh adopter."
workItemId: WI-397-release-v0-2-40
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-397-release-v0-2-40
capabilityClaims: [release_distribution, verification_performance, adopter_acceptance]
---

# WI-397 — v0.2.40 release and published performance inheritance

[简体中文](WI-397-release-v0-2-40.zh-CN.md) · [日本語](WI-397-release-v0-2-40.ja.md)

## Intent

Release v0.2.40 from the reviewed `main` so the WI-396 Rust clean-snapshot
fast path is available through the shared external Runtime. The release and
adopter checks must use immutable public artifacts and preserve repository
isolation; a source build is not installation evidence.

## Boundary

This Work Item advances the patch version, keeps the release workflow and
tri-language distribution documentation aligned, and runs the public Release
adopter and N-1 acceptance boundaries. It does not change governance semantics,
historical evidence, global Agent/MCP configuration, or the performance budget
to make a check pass. Each adopter binds the shared binary with an explicit
`--repo` and keeps its own `.ai/` state.

## Acceptance

1. Cargo metadata and lockfile advance exactly one patch from v0.2.39 to
   v0.2.40; no existing tag or Release is reused.
2. The reviewed workflow builds every declared target from the exact merged
   commit and binds the manifest, Formula, SHA256SUMS, target SBOM, provenance,
   and immutable tag/Release identity.
3. The downloaded public v0.2.40 binary is checksum-verified and its version,
   binary digest, platform, and Runtime identity are recorded without source
   or workspace fallback.
4. Public adopter acceptance proves attach/profile/Agent doctor, the
   `first-adopter-smoke` `not_ready` boundary, lifecycle and evidence reuse,
   isolation, cleanup, and repository/runtime identity.
5. Applicable N-1 acceptance preserves historical bytes and records
   `releasePublished: true` when a post-release check fails.
6. The current repository and fresh adopter inherit WI-396's measured
   clean-snapshot optimization without a global repository or cross-repository
   cache.
7. Reviewed merge, finalization, close, synchronization, and exact cleanup
   leave `main` `ready_on_base` with no open PR or `codex/*` branch.

## Verification boundary

Pre-release checks use the strict source and staged release gates. Post-release
checks download only the immutable public artifacts and persist runtime,
adopter, isolation, cleanup, and checksum evidence. The release does not claim
provider or enterprise performance from local measurements.
