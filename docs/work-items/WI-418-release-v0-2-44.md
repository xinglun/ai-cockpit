---
author: AI Cockpit maintainers
title: WI-418 — v0.2.44 release
description: Publish the reviewed Runtime with lockfile-aware Cargo verification selection.
workItemId: WI-418-release-v0-2-44
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-418-release-v0-2-44
---

# WI-418 — v0.2.44 release

[简体中文](WI-418-release-v0-2-44.zh-CN.md) · [日本語](WI-418-release-v0-2-44.ja.md)

## Intent

Publish the reviewed `main` as v0.2.44 after the lockfile-aware Cargo
verification command selection fix, while keeping release identity and the
three-language documentation synchronized.

## Boundary

This Work Item advances the patch release and validates the strict release
source route. It does not change governance semantics, copy the reference/V1
runtime or installer, modify global Agent/MCP configuration, or change adopter
application source. Public-artifact adopter acceptance remains a separate
post-release Work Item.

## Acceptance

- Cargo metadata and lockfile advance exactly one patch from v0.2.43 to v0.2.44;
  the tag and Release are not reused.
- The reviewed workflow binds the exact commit, target archives, SBOMs,
  manifest, Formula, checksums, provenance, and immutable tag/Release identity.
- Release, installation, versioning, and parity guidance stays synchronized in
  English, Simplified Chinese, and Japanese.
- A later isolated Work Item accepts only the immutable public v0.2.44 artifact.
- Reviewed merge, finalization, close, synchronization, and exact cleanup leave
  `main` `ready_on_base`.

## Verification boundary

Pre-release checks use the declared strict source and release gates. A staged
candidate or source build is never presented as public adopter evidence.
