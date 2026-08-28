---
author: AI Cockpit maintainers
title: "WI-374 — v0.2.39 release and exact verification reuse acceptance"
description: "Release the dynamic verification-reuse Runtime after repairing the recovered parity projection, then accept the public artifact in isolated repositories."
workItemId: WI-374-release-v0-2-39
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-374-release-v0-2-39
capabilityClaims: [release_distribution, verification_performance, adopter_acceptance]
---

# WI-374 — v0.2.39 release and exact verification reuse

[简体中文](WI-374-release-v0-2-39.zh-CN.md) · [日本語](WI-374-release-v0-2-39.ja.md)

## Intent

Publish v0.2.39 from the reviewed synchronized `main` so the dynamic,
identity-bound exact verification-reuse optimization is available to both this
repository and future adopter repositories. Repair the release-blocking parity
projection for the recovered WI-370 and WI-371 receipts before publication.

## Scope and boundary

- Align Cargo metadata, lockfile, versioning, release, and distribution
  documentation with v0.2.39 in all three supported languages.
- Reference the authoritative digest-suffixed recovery receipts in the three
  parity ledgers without rewriting predecessor evidence.
- Run the strict release workflow and publish only its immutable, checksummed,
  SBOM-bound, provenance-bound artifacts.
- Install only the downloaded public artifact into this repository and a fresh
  isolated adopter; retain runtime, repository-isolation, and exact-reuse
  evidence.

Runtime semantics, historical evidence rewrites, global Agent/MCP configuration,
source-build fallback, and a second technology-stack adopter are outside this
Work Item.

## Acceptance

1. Cargo metadata and lockfile report v0.2.39 consistently.
2. Recovered parity rows point to their authoritative recovery receipts and
   strict documentation/governance gates pass.
3. The public Release contains the target archives, manifest, SHA256SUMS,
   target-bound SBOMs, Formula, and provenance evidence.
4. The installed public binary's version and digest are bound in acceptance
   receipts; no source or workspace fallback is used.
5. Exact valid evidence is reused in this repository and a fresh adopter,
   while changed, stale, or unknown inputs rerun or stop fail-closed.
6. Acceptance proves HOME/XDG isolation, allowed runtime-write roots, cleanup,
   lifecycle evidence, and unchanged Release truth on failure.
7. Reviewed merge, finalization, close, synchronized default branch, and exact
   branch/worktree cleanup leave the repository `ready_on_base`.

## Verification boundary

Pre-release checks use the strict repository gate manifest and staged artifact
acceptance. Post-release checks download the immutable v0.2.39 artifact and
record its tag, archive digest, binary digest, platform, and source. The
optimization is exact-match reuse only: first or invalidated verification is
still executed, and the measured benefit is not generalized to those paths.
