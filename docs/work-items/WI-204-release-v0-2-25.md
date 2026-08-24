---
author: AI Cockpit maintainers
title: "WI-204 — v0.2.25 release and transition compatibility recovery"
description: "Repeat the v0.2.25 release boundary with resource context bound before verification and archive."
audience:
  - maintainer
  - adopter
workItemId: WI-204-release-v0-2-25
status: recovered
authority: canonical
lastVerifiedBy: WI-204-release-v0-2-25
---

# WI-204 — v0.2.25 release and transition compatibility recovery

WI-204 is the explicit successor of WI-203. WI-203 remains immutable history
because its archive was created before `finalize-plan` bound the real branch,
worktree, and pull request context. This Work Item repeats the same v0.2.25
release boundary with that context bound before verification and archive.

The public acceptance boundary uses only immutable v0.2.25 Release assets. It
records Release identity, downloaded adopter and N-1 receipts, isolation and
cleanup evidence, the append-only transition, and terminal human decisions.
The v0.2.24 tag remains failed pre-publication history and is not reused.

The tri-language entry points are:

- [简体中文](WI-204-release-v0-2-25.zh-CN.md)
- [日本語](WI-204-release-v0-2-25.ja.md)

## Acceptance boundary

1. Version, distribution documentation, and all parity rows agree on v0.2.25.
2. The immutable public Release provides complete manifest, checksums, archives,
   SBOM, Formula, and provenance evidence.
3. The downloaded v0.2.25 binary passes isolated adopter and v0.2.23→v0.2.25
   N-1 acceptance without source fallback.
4. Installed v0.2.25 accepts and records the append-only finalization
   transition.
5. WI-203 recovery, Runtime identity, evidence reuse, isolation, cleanup, and
   tri-language human Outcome remain auditable.
