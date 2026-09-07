---
author: AI Cockpit maintainers
title: WI-644 — v0.2.86 release
description: Publish the final Runtime release after the complete reference comparison and verify its public artifact boundaries.
audience: [adopter, maintainer, reviewer]
workItemId: WI-644-release-v0-2-86
status: implemented
authority: human-authorized
lastVerifiedBy: WI-644-release-v0-2-86
terminalArchive: .ai/work-items/archive/WI-644-release-v0-2-86.contract.json
terminalVerification: .ai/evidence/WI-644-release-v0-2-86.verification.json
terminalFinalization: .ai/decisions/WI-644-release-v0-2-86.finalize.json
terminalDecision: .ai/decisions/WI-644-release-v0-2-86.close.json
capabilityClaims: [release_distribution, reference_comparison, adopter_acceptance]
---

# WI-644 — v0.2.86 release

[简体中文](WI-644-release-v0-2-86.zh-CN.md) · [日本語](WI-644-release-v0-2-86.ja.md)

## Intent

Publish the final Runtime release after all six reference-comparison batches
are complete. The release binds one reviewed commit to version metadata,
checksums, SBOM/provenance, installation instructions, and downloaded-artifact
adopter acceptance.

## Boundary

This Work Item updates release/version projections and does not copy source
implementation or source wire formats. Public and N-1 adopter evidence is
created only by the release workflow from immutable published artifacts. Object
repositories remain independent consumers of the shared Runtime.

## Verification

Workspace tests, release policy, source archive, action runtime, adopter-wrapper,
reference inventory (5,175 records), tri-language documentation, and governance
integrity checks passed before the reviewed PR. The public tag and post-release
acceptance remain provider-bound evidence for the release workflow.

See the terminal records linked in the front matter for the authoritative
Contract, verification, finalization, and close evidence.
