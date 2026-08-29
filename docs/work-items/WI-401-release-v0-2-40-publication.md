---
author: AI Cockpit maintainers
title: "WI-401 — v0.2.40 public Release publication and adopter acceptance"
description: "Publish the reviewed v0.2.40 Runtime and accept its immutable artifact in a fresh adopter."
workItemId: WI-401-release-v0-2-40-publication
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-401-release-v0-2-40-publication
terminalArchive: .ai/work-items/archive/WI-401-release-v0-2-40-publication.contract.json
terminalVerification: .ai/evidence/WI-401-release-v0-2-40-publication.verification.json
terminalFinalization: .ai/decisions/WI-401-release-v0-2-40-publication.finalize.json
terminalDecision: .ai/decisions/WI-401-release-v0-2-40-publication.close.json
capabilityClaims: [release_distribution, adopter_acceptance, runtime_installation]
---

# WI-401 — v0.2.40 public Release publication and adopter acceptance

[简体中文](WI-401-release-v0-2-40-publication.zh-CN.md) · [日本語](WI-401-release-v0-2-40-publication.ja.md)

## Intent

Publish v0.2.40 from the reviewed synchronized `main`, then prove that the
immutable public artifact can govern a fresh adopter and this repository.

## Boundary

This Work Item covers only tag/Release publication, immutable artifact
acceptance, installation of the verified binary, and auditable external
acceptance evidence. It does not change Runtime semantics, reference parity,
business-project code, or global Agent/MCP configuration. Public acceptance
must reject source-build and workspace-binary fallback.

## Acceptance

1. The v0.2.40 tag and public Release are produced from the reviewed main
   merge and pass release identity, SBOM, provenance, and checksum gates.
2. The downloaded artifact records tag, version, archive digest, binary digest,
   platform, and download source, and those identities are bound into evidence.
3. A fresh adopter has an independent repository identity and complete
   lifecycle; `first-adopter-smoke` remains `not_ready`, evidence reuse is
   demonstrated, and forbidden roots plus temporary run roots are clean.
4. The verified public binary is installed for this repository and reports
   `COMPATIBLE` and `doctor=ok`; main ends synchronized and ready on base.

## Verification boundary

Post-release acceptance is evidence about the published artifact. It cannot
rewrite Release truth or historical evidence, and a failed acceptance records
`releasePublished: true` with `adopterAcceptance: failed`.
