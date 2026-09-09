---
author: AI Cockpit maintainers
title: WI-764 — v0.2.88 release
description: Publish the next Runtime release and verify immutable public artifacts and adopter boundaries.
audience: [adopter, maintainer, reviewer]
workItemId: WI-764-release-v0-2-88
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-764-release-v0-2-88
terminalArchive: .ai/work-items/archive/WI-764-release-v0-2-88.contract.json
terminalVerification: .ai/evidence/WI-764-release-v0-2-88.verification.json
terminalFinalization: .ai/decisions/WI-764-release-v0-2-88.finalize.json
terminalDecision: .ai/decisions/WI-764-release-v0-2-88.close.json
capabilityClaims: [release_distribution, reference_comparison, adopter_acceptance]
---

# WI-764 — v0.2.88 release

[简体中文](WI-764-release-v0-2-88.zh-CN.md) · [日本語](WI-764-release-v0-2-88.ja.md)

## Intent

Publish v0.2.88 after the completed performance measurement Work Items and
verify the immutable public release artifact, checksums, SBOM/provenance,
installation boundary, isolated adopter acceptance, and N-1 upgrade boundary.

## Boundary

This Work Item updates the Runtime package version and current release
documentation. It does not change Runtime behavior, copy the reference source,
rewrite historical governance or release bytes, modify global Agent/MCP
configuration, pre-create a provider Release, or reuse a reserved tag.
Release acceptance uses only the immutable public artifact; a source checkout
or workspace binary is not a release substitute.

## Verification

The reviewed PR, source quality, version consistency, release policy, five
target builds, manifest/checksums, SBOM/provenance, platform smoke, staged
adopter acceptance, public adopter acceptance, and N-1 upgrade checks must
pass. The installed downloaded binary must report v0.2.88, match the public
manifest and digest, and leave the repository `ready_on_base`.

See the terminal records linked in the front matter after close for the
authoritative Contract, verification, finalization, and decision evidence.
