---
author: AI Cockpit maintainers
title: "WI-415 — v0.2.43 release"
description: "Publish the reviewed post-WI-414 Runtime and establish the next public-artifact acceptance baseline."
workItemId: WI-415-release-v0-2-43
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-415-release-v0-2-43
capabilityClaims: [release_distribution, repository_isolation]
sourceCommit: 107dfab6e6e331041a73fce7406f573bfbd7610c
---

# WI-415 — v0.2.43 release

[简体中文](WI-415-release-v0-2-43.zh-CN.md) · [日本語](WI-415-release-v0-2-43.ja.md)

## Intent

Publish v0.2.43 from the reviewed post-WI-414 `main` and leave a clean,
reviewed base for the separate immutable public adopter-acceptance step.

## Boundary

This Work Item advances the patch version, synchronizes current release,
installation, versioning, and parity guidance in all three languages, and
validates the strict release source route. It does not change Runtime
governance semantics, historical evidence, global Agent/MCP configuration, or
adopter application source. Public-artifact adopter acceptance remains a
separate post-release Work Item.

## Acceptance

1. Cargo metadata and lockfile advance exactly one patch from v0.2.42 to
   v0.2.43 without reusing an existing tag or Release.
2. The reviewed release workflow binds the exact reviewed commit, target
   archives, SBOMs, manifest, Formula, SHA256SUMS, provenance, and immutable
   tag/Release identity.
3. Current release, installation, versioning, and parity documentation remain
   synchronized in English, Simplified Chinese, and Japanese, with historical
   releases explicitly retained as historical.
4. Post-release acceptance uses only the immutable public v0.2.43 artifact in
   an isolated Work Item; no source or workspace fallback is allowed.
5. Reviewed merge, finalization, close, default-branch synchronization, and
   exact branch/worktree cleanup leave `main` `ready_on_base`.

## Verification boundary

Pre-release checks use the declared strict source and release gates. This Work
Item must not present a staged candidate or source build as public adopter
evidence. Any release or cleanup failure remains visible and cannot rewrite
published Release truth.

[English](WI-415-release-v0-2-43.md) · [简体中文](WI-415-release-v0-2-43.zh-CN.md) · [日本語](WI-415-release-v0-2-43.ja.md)
