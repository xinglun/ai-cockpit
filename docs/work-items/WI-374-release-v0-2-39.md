---
author: AI Cockpit maintainers
title: "WI-374 — v0.2.39 release and exact verification reuse acceptance"
description: "Release the dynamic verification-reuse Runtime after repairing the recovered parity projection, then accept the public artifact in isolated repositories."
workItemId: WI-374-release-v0-2-39
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-374-release-v0-2-39
terminalArchive: .ai/work-items/archive/WI-374-release-v0-2-39.contract.json
terminalVerification: .ai/evidence/WI-374-release-v0-2-39.verification.json
terminalFinalization: .ai/decisions/WI-374-release-v0-2-39.finalize.json
terminalDecision: .ai/decisions/WI-374-release-v0-2-39.close.json
capabilityClaims: [release_distribution, verification_performance, adopter_acceptance]
---

# WI-374 — v0.2.39 release and exact verification reuse

[简体中文](WI-374-release-v0-2-39.zh-CN.md) · [日本語](WI-374-release-v0-2-39.ja.md)

## Intent

Prepare v0.2.39 from the reviewed synchronized `main` so the dynamic,
identity-bound exact verification-reuse optimization is ready for publication.
Repair the release-blocking parity projection for the recovered WI-370 and
WI-371 receipts before publication. Public-artifact and adopter acceptance are
explicitly handed to successor WI-376 after the immutable tag exists.

## Scope and boundary

- Align Cargo metadata, lockfile, versioning, release, and distribution
  documentation with v0.2.39 in all three supported languages.
- Reference the authoritative digest-suffixed recovery receipts in the three
  parity ledgers without rewriting predecessor evidence.
- Run the strict release-policy and staged checks needed before the immutable
  tag; do not claim a public artifact before publication.
- Preserve a post-release handoff so successor WI-376 can install only the
  downloaded public artifact into this repository and a fresh isolated adopter.

Runtime semantics, historical evidence rewrites, global Agent/MCP configuration,
source-build fallback, and a second technology-stack adopter are outside this
Work Item.

## Acceptance

1. Cargo metadata and lockfile report v0.2.39 consistently.
2. Recovered parity rows point to their authoritative recovery receipts and
   strict documentation/governance gates pass.
3. Public Release asset, checksum, SBOM, Formula, and provenance acceptance is
   explicitly deferred to successor WI-376 and not claimed here.
4. Downloaded public-binary identity and no-fallback acceptance are explicitly
   deferred to successor WI-376.
5. Exact-reuse and fresh-adopter acceptance are explicitly deferred to
   successor WI-376.
6. Isolation, cleanup, lifecycle, and Release-truth failure handling are
   explicitly deferred to successor WI-376.
7. Reviewed merge, finalization, close, synchronized default branch, and exact
   branch/worktree cleanup leave the repository `ready_on_base`.

## Verification boundary

Pre-release checks use the strict repository gate manifest and staged checks.
Post-release checks are the successor WI-376 boundary: they download the
immutable v0.2.39 artifact and record its tag, archive digest, binary digest,
platform, and source. The optimization is exact-match reuse only: first or
invalidated verification is still executed, and the measured benefit is not
generalized to those paths.
