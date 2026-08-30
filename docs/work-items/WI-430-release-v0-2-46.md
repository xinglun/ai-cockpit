---
author: AI Cockpit maintainers
title: "WI-430 — v0.2.46 release"
description: Publish the WI-429 historical-recovery fix as an immutable Runtime release.
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
workItemId: WI-430-release-v0-2-46
lastVerifiedBy: WI-430-release-v0-2-46
---

# WI-430 — v0.2.46 release

[简体中文](WI-430-release-v0-2-46.zh-CN.md) · [日本語](WI-430-release-v0-2-46.ja.md)

## Intent

Publish the reviewed WI-429 recovery-history fix as v0.2.46 so adopters can
install the corrected Runtime from an immutable public artifact.

## Boundary

This Work Item advances one patch release and synchronizes release guidance.
It does not alter governance semantics, copy the reference/V1 runtime, modify
global Agent or MCP configuration, or perform adopter application changes.

## Acceptance

- Cargo metadata and lockfile advance exactly one patch from v0.2.45 to v0.2.46;
  no existing tag or Release is reused.
- The reviewed release workflow binds the exact commit, five target archives,
  SBOMs, manifest, Formula, checksums, provenance, and Release identity.
- Release, installation, versioning, and parity guidance stay synchronized in
  English, Simplified Chinese, and Japanese; v0.2.45 remains historical.
- Post-release acceptance uses only the immutable public v0.2.46 artifact and
  rejects source, workspace, or local-binary fallback.
- Reviewed merge, finalization, close, synchronization, and exact cleanup leave
  `main` `ready_on_base`.

## Verification boundary

Pre-release checks use the Contract-declared release gates. Public adopter
acceptance is a separate post-release Work Item and must retain runtime
identity, artifact digests, isolation manifests, and cleanup proof.
