---
author: AI Cockpit maintainers
workItemId: WI-126-status-outcome
title: Read-only Work Item status and human handoff projection
description: Expose one evidence-bound status and Outcome projection to CLI and MCP.
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: documentation-acceptance
---

# WI-126 — Read-only status and human handoff

This Work Item adds a request-scoped `work-item status` projection and keeps
CLI/MCP human Outcome delivery on one validated source. It does not create a
scheduler, a global current project, or a second governance decision engine.

Delivered boundaries:

- lifecycle phase, governance state, activity health, fact counts, blockers,
  evidence, risks, permissions, unknowns, diagnostics, and source digests;
- a direct first-line `Outcome: 🔴/🟡/🟢` human handoff in CLI and MCP;
- Contract-language acceptance criteria preserved byte-for-byte and labeled;
- historical, missing, stale, malformed, foreign, and symlink evidence remain
  non-green and are not rewritten;
- three-language Contract field mapping and reference-parity baseline now include
  WI-125.

The final twenty-dimension aggregator and external assurance remain a later
boundary. Status and Outcome are read-only projections, not authorization.
