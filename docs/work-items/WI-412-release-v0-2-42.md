---
author: AI Cockpit maintainers
title: "WI-412 — v0.2.42 release preparation"
description: "Publish the reviewed post-WI-411 Runtime and preserve a clean base for public adopter acceptance."
workItemId: WI-412-release-v0-2-42
audience: [adopter, maintainer, reviewer]
status: recovered
authority: human-authorized
lastVerifiedBy: WI-412-release-v0-2-42
capabilityClaims: [release_distribution, repository_isolation]
---

# WI-412 — v0.2.42 release preparation

[简体中文](WI-412-release-v0-2-42.zh-CN.md) · [日本語](WI-412-release-v0-2-42.ja.md)

## Intent

Publish v0.2.42 from the reviewed post-WI-411 `main` and leave a clean,
reviewed base for the separate immutable public adopter-acceptance step.

## Boundary

This Work Item advances the patch version, synchronizes the current release
and versioning guidance in all three languages, validates the strict release
source route, and records the reviewed lifecycle. It does not change Runtime
governance semantics, historical evidence, global Agent/MCP configuration, or
adopter application source. Public-artifact adopter acceptance is a separate
post-release Work Item and is not claimed here.

## Acceptance

1. Cargo metadata and lockfile advance exactly one patch from v0.2.41 to
   v0.2.42 without reusing an existing tag or Release.
2. The reviewed release workflow binds the exact reviewed commit, target
   archives, SBOMs, manifest, Formula, SHA256SUMS, provenance, and immutable
   tag/Release identity.
3. Current release, installation, versioning, and parity documentation remain
   synchronized in English, 简体中文, and 日本語, with historical releases kept
   explicitly historical.
4. Post-release acceptance uses only the immutable public v0.2.42 artifact in
   a separate isolated Work Item; no source or workspace fallback is allowed.
5. Reviewed merge, finalization, close, default-branch synchronization, and
   exact branch/worktree cleanup leave `main` `ready_on_base`.

## Verification boundary

Pre-release checks use the declared strict source and release gates. This
Work Item must not present a staged candidate or source build as public
adopter evidence. Any release or cleanup failure remains visible and cannot
rewrite published Release truth.
