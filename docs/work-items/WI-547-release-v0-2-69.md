---
author: AI Cockpit maintainers
title: "WI-547 — v0.2.69 release and public-artifact acceptance"
description: "Correct the failed v0.2.68 publication projection and publish the next immutable runtime baseline."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-547-release-v0-2-69
lastVerifiedBy: WI-547-release-v0-2-69
terminalArchive: .ai/work-items/archive/WI-547-release-v0-2-69.contract.json
terminalVerification: .ai/evidence/WI-547-release-v0-2-69.verification.json
terminalFinalization: .ai/decisions/WI-547-release-v0-2-69.finalize.json
terminalDecision: .ai/decisions/WI-547-release-v0-2-69.close.json
---

[简体中文](WI-547-release-v0-2-69.zh-CN.md) · [日本語](WI-547-release-v0-2-69.ja.md)

# WI-547 — v0.2.69 release and public-artifact acceptance

## Objective

Publish a truthful v0.2.69 Runtime baseline from the reviewed default branch.
The failed v0.2.68 tag remains immutable history and is never presented as
public or installable.

## Scope and boundary

- Package identity and lockfile.
- Three-language release, versioning, and distribution documentation.
- Work Item and reference-parity projections for this release.
- Public artifact, checksum, SBOM, adopter, and installation acceptance are
  release evidence bound to this Work Item.
- Runtime behavior, object repositories, global Agent/MCP configuration, and
  the failed v0.2.68 tag are outside the change scope.

## Acceptance

1. Package and documentation identities consistently declare v0.2.69, while
   v0.2.68 is explicitly documented as failed publication history.
2. Release CI and policy gates produce a manifest, SHA256SUMS, SBOM, provenance,
   and public assets bound to the immutable tag and reviewed source commit.
3. Download-only public-binary acceptance passes in isolated roots, proves
   cleanup and forbidden-write isolation, and the same verified binary is
   installed for explicit-repository self-health checks.

## Verification boundary

Contract acceptance prose remains authoritative and in its original language;
localized headings do not translate governance facts. Release evidence must
bind the published tag, archive digest, binary digest, runtime identity, and
adopter receipt. A failed publication is recorded as failed and never reused.
