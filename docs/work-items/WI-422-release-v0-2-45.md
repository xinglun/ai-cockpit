---
author: AI Cockpit maintainers
title: WI-422 — v0.2.45 release
description: Publish the reviewed Runtime after the mixed-monorepo reference batch.
workItemId: WI-422-release-v0-2-45
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-422-release-v0-2-45
terminalArchive: .ai/work-items/archive/WI-422-release-v0-2-45.contract.json
terminalVerification: .ai/evidence/WI-422-release-v0-2-45.verification.json
terminalFinalization: .ai/decisions/WI-422-release-v0-2-45.finalize.fea37841b1ed548581e851a7e36b2b0128ee19526142210f2eef0a5d5f5a9198.json
terminalDecision: .ai/decisions/WI-422-release-v0-2-45.close.json
---

# WI-422 — v0.2.45 release

[简体中文](WI-422-release-v0-2-45.zh-CN.md) · [日本語](WI-422-release-v0-2-45.ja.md)

## Intent

Publish the reviewed `main` as v0.2.45 after the mixed-monorepo reference
comparison batch, while keeping release identity, installation guidance, and
the three-language parity record synchronized.

## Boundary

This Work Item advances one patch release and verifies the existing strict
release route. It does not change Runtime governance semantics, copy reference
or V1 runtime/installer code, modify global Agent/MCP configuration, or include
adopter application source. Public-artifact adopter acceptance remains a
separate post-release Work Item and must use only the immutable v0.2.45 asset.

## Acceptance

- Cargo metadata and lockfile advance exactly one patch from v0.2.44 to v0.2.45;
  no existing tag or Release is reused.
- The reviewed workflow binds the exact reviewed commit, target archives, SBOMs,
  manifest, Formula, checksums, provenance, and immutable tag/Release identity.
- Release, installation, versioning, and parity guidance is synchronized in
  English, Simplified Chinese, and Japanese, while v0.2.44 remains historical
  evidence until the new public adopter baseline is accepted.
- A separate isolated post-release Work Item accepts only the immutable public
  v0.2.45 artifact; source, workspace, or local binaries are forbidden.
- Reviewed merge, finalization, close, default-branch synchronization, and
  exact branch/worktree cleanup leave `main` `ready_on_base`.

## Verification boundary

Pre-release checks use the Contract-declared strict source and release gates.
Staged candidates and source builds are not public adopter evidence. The
post-release acceptance receipt must retain runtime identity, artifact digests,
isolation manifests, and cleanup proof without rewriting Release truth.
