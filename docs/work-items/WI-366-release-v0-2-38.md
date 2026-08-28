---
author: AI Cockpit maintainers
title: "WI-366 — v0.2.38 release preparation after N-1 identity root-fix"
workItemId: WI-366-release-v0-2-38
description: "Prepare the first release after the v0.2.37 N-1 Git identity root-fix and hand off immutable public-artifact acceptance."
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-366-release-v0-2-38
terminalArchive: .ai/work-items/archive/WI-366-release-v0-2-38.contract.json
terminalVerification: .ai/evidence/WI-366-release-v0-2-38.verification.json
terminalFinalization: .ai/decisions/WI-366-release-v0-2-38.finalize.json
terminalDecision: .ai/decisions/WI-366-release-v0-2-38.close.json
capabilityClaims: [release_distribution, adopter_acceptance]
---

# WI-366 — v0.2.38 release preparation after N-1 identity root-fix

[简体中文](WI-366-release-v0-2-38.zh-CN.md) · [日本語](WI-366-release-v0-2-38.ja.md)

## Intent

Prepare v0.2.38 from the reviewed, synchronized `main` after the v0.2.37
N-1 upgrade acceptance root-fix. Public artifact installation and adopter
acceptance are a post-release successor boundary and are not claimed here.

## Scope and boundary

- Align workspace package metadata, lockfile, and current tri-language
  release/versioning documentation to v0.2.38.
- Run the reviewed release policy, documentation, parity, and staged adopter
  regression checks before tagging.
- Preserve an explicit handoff for the successor Work Item that will download
  the immutable public artifact and run adopter/N-1 acceptance after release.
- Preserve the unpublished v0.2.37 candidate failure as immutable history;
  do not move or relabel its tag.

Runtime behavior changes, historical evidence rewrites, global Agent/MCP
configuration, source-build fallbacks, and a second technology-stack adopter
are outside this Work Item.

## Acceptance

1. Cargo metadata and lockfile report 0.2.38 consistently.
2. Reviewed CI and release policy checks pass before the immutable tag.
3. The v0.2.37 N-1 Git identity failure is covered by a repository-local
   identity regression with no global Git configuration requirement.
4. The post-release public artifact, installed binary, adopter isolation, and
   N-1 acceptance are explicitly handed off to a successor Work Item and are
   not claimed before publication.
5. Merge, finalization, close, and exact branch/worktree cleanup leave the
   repository ready on the synchronized default branch.
6. Closed WI-365 tri-language Work Item projections report implemented status
   consistently with their terminal evidence and parity rows.

## Verification boundary

The Runtime lifecycle records the Contract, checkpoint, verification, archive,
finalization, and close evidence. A successor Work Item bound to the immutable
published tag must record hosted workflow, install, and adopter receipts before
any public-artifact claim. The failed v0.2.37 candidate remains unchanged and
is not an install source.
