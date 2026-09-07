---
author: AI Cockpit maintainers
title: WI-646 — v0.2.87 release
description: Publish the Runtime release and verify its immutable public artifact and adopter boundaries.
audience: [adopter, maintainer, reviewer]
workItemId: WI-646-release-v0-2-87
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-646-release-v0-2-87
capabilityClaims: [release_distribution, reference_comparison, adopter_acceptance]
---

# WI-646 — v0.2.87 release

[简体中文](WI-646-release-v0-2-87.zh-CN.md) · [日本語](WI-646-release-v0-2-87.ja.md)

## Intent

Publish v0.2.87 after the Runtime-version binding correction and verify the
public release artifact, checksums, SBOM/provenance, installation path, and
isolated adopter acceptance.

## Boundary

This Work Item updates the Runtime version and release/documentation
projections. It does not copy the reference source, rewrite historical
governance bytes, operate an object repository, or modify global Agent/MCP
configuration. Release acceptance must use only the immutable public artifact;
the source workspace and local target binary are not release substitutes.

## Verification

The declared workspace, documentation, release-policy, source-quality,
published-adopter, and N-1 upgrade checks must pass. The installed published
binary must report v0.2.87 and its downloaded digest must match the public
release manifest and receipt. The repository must return to `ready_on_base`.

See the terminal records linked in the front matter after close for the
authoritative Contract, verification, finalization, and decision evidence.
