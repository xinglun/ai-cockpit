---
author: AI Cockpit maintainers
title: "WI-591 — v0.2.77 release and object-adopter recovery handoff"
description: "Publish the Runtime release that contains the predecessor-close revalidation fix and validate its public artifacts."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-591-release-v0-2-77
lastVerifiedBy: WI-591-release-v0-2-77
terminalArchive: .ai/work-items/archive/WI-591-release-v0-2-77.contract.json
terminalVerification: .ai/evidence/WI-591-release-v0-2-77.verification.json
terminalFinalization: .ai/decisions/WI-591-release-v0-2-77.finalize.5e0a83694b8c0cc446f933fdfec8909a4fe84d4bcceb1e55ba03d5a2fbe6e7aa.json
terminalDecision: .ai/decisions/WI-591-release-v0-2-77.close.json
---

[简体中文](WI-591-release-v0-2-77.zh-CN.md) · [日本語](WI-591-release-v0-2-77.ja.md)

# WI-591 — v0.2.77 release and object-adopter recovery handoff

## Objective

Publish v0.2.77 from the reviewed, synchronized default branch. The release
must include the Contract-amendment predecessor-close revalidation fix from
WI-589, retain immutable release evidence, and provide a read-only acceptance
handoff for the object repository.

## Boundary

This Work Item changes package version metadata and release documentation only.
Runtime implementation, object repositories, global Agent/MCP configuration,
historical evidence bytes, and reference-source implementation are outside the
boundary. Public adopter and N-1 acceptance are post-release evidence and must
use downloaded immutable artifacts, never a source checkout or workspace build.

## Acceptance

1. Cargo metadata, lockfile, and the English/Chinese/Japanese release and
   versioning guides identify v0.2.77 and retain v0.2.76 as the preceding
   public baseline.
2. Release policy checks prove the annotated tag, five target artifacts,
   checksums, SBOM/provenance, and Runtime identity are bound to one source
   commit.
3. Post-release adopter and N-1 harnesses use only the public v0.2.77
   artifacts, prove forbidden-root isolation, and prove temporary-run cleanup.
4. The object repository remains untouched; its team receives exact
   compatibility, recovery, and revalidation commands after publication.
5. A release or adopter failure preserves published truth and records a
   failure receipt; no failed tag or historical evidence is rewritten.

## Verification

Run the release policy, documentation, parity, and locked workspace checks
listed in the Contract. After publication, run the public adopter and N-1
acceptance harnesses against v0.2.77 and record their immutable receipts.
