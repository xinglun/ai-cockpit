---
author: AI Cockpit maintainers
title: "WI-205 — v0.2.25 release and transition compatibility recovery"
description: "Re-establish the v0.2.25 release boundary from the synchronized default branch after predecessor base identity drift."
audience:
  - maintainer
  - adopter
workItemId: WI-205-release-v0-2-25
status: in_progress
authority: canonical
lastVerifiedBy: WI-205-release-v0-2-25
---

# WI-205 — v0.2.25 release and transition compatibility recovery

WI-205 succeeds WI-204 because the predecessor was started from an open
predecessor branch and could not truthfully bind the actual pull-request base.
The predecessor archive and failed finalization attempt remain immutable. This
Work Item records the synchronized `origin/main` base before verification and
archive, then completes the v0.2.25 public release boundary.

Only immutable v0.2.25 Release assets are valid for adopter acceptance. The
receipt must bind Release identity, downloaded binary and N-1 evidence,
isolated-root manifests, cleanup proof, the append-only transition, and
terminal human decisions. v0.2.24 remains failed pre-publication history.

The tri-language entry points are:

- [简体中文](WI-205-release-v0-2-25.zh-CN.md)
- [日本語](WI-205-release-v0-2-25.ja.md)

## Acceptance boundary

1. v0.2.25 version, documentation, and parity are consistent.
2. The immutable public Release has complete manifest, checksums, archives,
   SBOM, Formula, and provenance evidence.
3. The downloaded v0.2.25 binary passes isolated adopter and v0.2.23→v0.2.25
   N-1 acceptance without source fallback.
4. Installed v0.2.25 accepts and records the append-only finalization
   transition.
5. WI-204 recovery, base and Runtime identity, evidence reuse, isolation,
   cleanup, and tri-language Outcome are auditable.
