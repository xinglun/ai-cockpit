---
author: AI Cockpit maintainers
title: "WI-203 — v0.2.25 release and transition compatibility"
description: "Recover the failed v0.2.24 publication attempt and establish a fresh immutable v0.2.25 release baseline."
audience:
  - maintainer
  - adopter
workItemId: WI-203-release-v0-2-25
status: recovered
authority: canonical
lastVerifiedBy: WI-203-release-v0-2-25
---

# WI-203 — v0.2.25 release and transition compatibility

This Work Item is the explicit successor of WI-202. The v0.2.24 tag and its
failed pre-publication workflow remain immutable history and are not reused.
The current baseline advances exactly one patch to v0.2.25.

Scope is limited to version/distribution documentation, parity and governance
records, public Release evidence, downloaded adopter acceptance, and the
installed Runtime finalization transition. Runtime source and CI workflow
implementation are out of scope.

The public acceptance boundary uses only immutable v0.2.25 Release assets. It
must record the Release manifest, archive and binary digests, adopter and N-1
receipts, isolated-root manifests, cleanup proof, transition receipt, and
terminal Human Decision. No source checkout or workspace binary is a valid
fallback.

The tri-language entry points are:

- [简体中文](WI-203-release-v0-2-25.zh-CN.md)
- [日本語](WI-203-release-v0-2-25.ja.md)

## Acceptance boundary

1. Version, current distribution documentation, and all parity rows agree on
   v0.2.25 before verification.
2. The public Release is stable, immutable, and has the complete manifest,
   checksums, archives, SBOM, Formula, and provenance evidence.
3. Downloaded v0.2.25 passes isolated adopter and v0.2.23→v0.2.25 N-1
   acceptance without source fallback.
4. Installed v0.2.25 accepts and records the append-only finalization
   transition.
5. WI-202 recovery, release identity, evidence reuse, isolation, cleanup, and
   tri-language human Outcome are auditable.
