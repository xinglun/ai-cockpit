---
author: AI Cockpit maintainers
title: "WI-359 — v0.2.36 release from synchronized main"
workItemId: WI-359-release-v0-2-36
description: "Publish the cleanup fix only from the fully synchronized default branch and verify the exact public artifact."
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-359-release-v0-2-36
terminalArchive: .ai/work-items/archive/WI-359-release-v0-2-36.contract.json
terminalVerification: .ai/evidence/WI-359-release-v0-2-36.verification.json
terminalFinalization: .ai/decisions/WI-359-release-v0-2-36.finalize.json
terminalDecision: .ai/decisions/WI-359-release-v0-2-36.close.json
capabilityClaims: [release_distribution, adopter_acceptance]
---

# WI-359 — v0.2.36 release from synchronized main

[简体中文](WI-359-release-v0-2-36.zh-CN.md) · [日本語](WI-359-release-v0-2-36.ja.md)

## Intent

Publish the cleanup fix as v0.2.36 from the reviewed, merged, and synchronized
default branch. The failed v0.2.35 publication remains immutable history.

## Scope

- Align workspace package, lockfile, and tri-language release/versioning docs to v0.2.36.
- Tag only the synchronized main revision that includes WI-358 finalization and close records.
- Use the hosted release workflow and its downloaded public artifact, checksum, SBOM,
  provenance, adopter, and cleanup evidence.
- Install the exact public macOS ARM64 binary and run explicit-repository health checks.

## Boundary

Do not move, delete, or relabel v0.2.35; do not rewrite its failed workflow truth.
Do not add runtime behavior, alter global Agent/MCP configuration, or use a source-build
fallback for release acceptance.

## Acceptance

1. All workspace packages and `Cargo.lock` resolve to 0.2.36 and pass version consistency.
2. v0.2.36 is tagged only after reviewed merge and synchronized default-branch checks.
3. The public workflow passes strict source quality, all target builds, artifact binding,
   and adopter acceptance with temporary-root cleanup proof.
4. The downloaded public binary checksum and digest match its release manifest; the installed
   binary reports 0.2.36 and passes inspect/status/doctor/agent doctor with explicit `--repo`.
5. Before the v0.2.36 tag is created, every merged delivery branch and worktree is synchronized
   and exactly cleaned; no stale merged branch or checkout remains.
6. v0.2.35 remains recorded as failed publication history and is not presented as a Release.

## Verification

The Runtime lifecycle evidence, reviewed PR, hosted release workflow, public Release
manifest/checksums, installed-binary digest, and adopter acceptance receipt are authoritative.
Terminal lifecycle: archive `.ai/work-items/archive/WI-359-release-v0-2-36.contract.json`;
verification `.ai/evidence/WI-359-release-v0-2-36.verification.json`; finalization
`.ai/decisions/WI-359-release-v0-2-36.finalize.json`; close
`.ai/decisions/WI-359-release-v0-2-36.close.json`.
