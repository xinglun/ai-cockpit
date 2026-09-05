---
author: AI Cockpit maintainers
title: "WI-596 — v0.2.78 release and object-adopter recovery handoff"
description: "Publish the Runtime release containing the archived-work-item recovery compatibility fix and validate its public artifacts."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-596-release-v0-2-78
lastVerifiedBy: WI-596-release-v0-2-78
terminalArchive: .ai/work-items/archive/WI-596-release-v0-2-78.contract.json
terminalVerification: .ai/evidence/WI-596-release-v0-2-78.verification.json
terminalFinalization: .ai/decisions/WI-596-release-v0-2-78.finalize.json
terminalDecision: .ai/decisions/WI-596-release-v0-2-78.close.json
---

[简体中文](WI-596-release-v0-2-78.zh-CN.md) · [日本語](WI-596-release-v0-2-78.ja.md)

# WI-596 — v0.2.78 release and object-adopter recovery handoff

## Objective

Publish v0.2.78 from the reviewed, synchronized default branch. This patch
exposes the already reviewed Contract-amendment predecessor-close recovery
fix, keeps the failed v0.2.77 tag as immutable history, and provides a
reproducible public-artifact acceptance handoff for adopter repositories.

## Boundary

This Work Item changes package version metadata and release documentation only.
Runtime source behavior, object repositories, global Agent/MCP configuration,
historical evidence bytes, and reference-source implementation are outside the
boundary. Public adopter and N-1 acceptance are post-release evidence and must
use downloaded immutable artifacts, never a source checkout or workspace build.

## Acceptance

1. Cargo metadata and lockfile resolve to v0.2.78; v0.2.77 remains failed
   unpublished history and is never retagged or used as an install baseline.
2. Release policy checks bind the annotated tag, five target artifacts,
   checksums, SBOM/provenance, and Runtime identity to one reviewed commit.
3. Public adopter and N-1 harnesses use only v0.2.78 artifacts, prove forbidden
   root isolation, and prove temporary-run cleanup on success and failure.
4. The object repository remains untouched; its team receives exact
   compatibility, recovery, and revalidation commands after publication.
5. Release or adopter failure preserves published truth and records a failure
   receipt without rewriting a failed tag or historical evidence.
6. English, Simplified Chinese, and Japanese release/versioning documentation
   agrees on the current public baseline and installation commands.

## Verification

Run the Contract-declared locked workspace, documentation, parity, release
policy, staged acceptance, and post-release public-artifact checks. Complete
the lifecycle only after reviewed PR checks pass, the v0.2.78 Release is
published, adopter/N-1 receipts are retained, and exact branch/worktree
cleanup is verified.
