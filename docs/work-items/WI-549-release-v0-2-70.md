---
author: AI Cockpit maintainers
title: "WI-549 — v0.2.70 release and public-artifact acceptance"
description: "Publish the next immutable Runtime baseline and verify it from downloaded public artifacts."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-549-release-v0-2-70
lastVerifiedBy: WI-549-release-v0-2-70
terminalArchive: .ai/work-items/archive/WI-549-release-v0-2-70.contract.json
terminalVerification: .ai/evidence/WI-549-release-v0-2-70.verification.json
terminalFinalization: .ai/decisions/WI-549-release-v0-2-70.finalize.json
terminalDecision: .ai/decisions/WI-549-release-v0-2-70.close.json
---

[简体中文](WI-549-release-v0-2-70.zh-CN.md) · [日本語](WI-549-release-v0-2-70.ja.md)

# WI-549 — v0.2.70 release and public-artifact acceptance

## Objective

Publish v0.2.70 from the reviewed default branch as the next immutable Runtime
baseline. The failed v0.2.68 tag remains historical and is never reused.

## Scope and boundary

- Update the workspace package identity and lockfile to v0.2.70.
- Keep English, Simplified Chinese, and Japanese release/versioning guidance in
  agreement, including tag, checksum, SBOM, provenance, attestation, and
  adopter-acceptance procedures.
- Bind the release to the closed WI-548 comparison batch and its promoted
  reference-parity record.
- Keep object repositories, global Agent/MCP configuration, source-template
  copying, and Runtime behavior changes outside this Work Item.

## Release acceptance

1. Package metadata, lockfile, and release documentation identify v0.2.70;
   v0.2.68 remains immutable failed-publication history.
2. Release CI produces public artifacts with release manifest, SHA256SUMS,
   SBOM, provenance, attestation, and tag/source/runtime identity bindings.
3. Post-release adopter and N-1 harnesses use only downloaded public artifacts
   in isolated roots, prove forbidden-root isolation and cleanup, and retain
   auditable receipts.
4. The same published binary can inspect, status, doctor, and govern this
   repository with an explicit repository context.

## Verification boundary

Contract acceptance prose remains authoritative in its original language.
Localized pages change presentation labels only; they do not translate or
alter governance facts. A failed publication is recorded as failed and its tag
is never reused. Object-repository acceptance is an external read-only step and
is not claimed by this Work Item until its team supplies a receipt.
