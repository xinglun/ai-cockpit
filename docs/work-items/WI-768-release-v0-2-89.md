---
author: AI Cockpit maintainers
title: WI-768 — v0.2.89 release recovery
description: Repair the strict release quality boundary and publish a new immutable release after the preserved v0.2.88 failure.
audience: [adopter, maintainer, reviewer]
workItemId: WI-768-release-v0-2-89
status: implemented
authority: human-authorized
lastVerifiedBy: WI-768-release-v0-2-89
terminalArchive: .ai/work-items/archive/WI-768-release-v0-2-89.contract.json
terminalVerification: .ai/evidence/WI-768-release-v0-2-89.verification.json
terminalFinalization: .ai/decisions/WI-768-release-v0-2-89.finalize.86a820e81c9c76239ad451ff19b9e778b0384f32ca9aae60af518f936bac8280.json
terminalDecision: .ai/decisions/WI-768-release-v0-2-89.close.json
capabilityClaims: [release_distribution, adopter_acceptance, governance_evidence]
---

[简体中文](WI-768-release-v0-2-89.zh-CN.md) · [日本語](WI-768-release-v0-2-89.ja.md)

# WI-768 — v0.2.89 release recovery

## Intent

Recover the failed v0.2.88 publication path without rewriting its immutable
history, then publish v0.2.89 with a release workflow that binds the active
Contract-aware Rust gate to the repository gate receipt.

## Boundary

This Work Item changes only the release workflow quality binding, Runtime
package version, current release/version documentation, and the evidence needed
to validate the public artifact and adopter boundary. Runtime production
behavior, performance implementation, historical WI-764/v0.2.88 bytes, global
Agent/MCP configuration, tag reuse, and pre-created provider Releases are out
of scope.

## Verification

The reviewed PR hosted checks and the v0.2.89 release workflow must prove the
Contract-aware source-quality receipt, repository gate receipt, five target
archives, SBOM/provenance, manifest/checksums, and release identity. Public
adopter installation and v0.2.87-to-v0.2.89 upgrade acceptance must use only
downloaded immutable artifacts and retain isolation/cleanup evidence.

Terminal Contract, verification, release, finalization, cleanup, and decision
paths are added here after the Work Item completes its Runtime lifecycle.
