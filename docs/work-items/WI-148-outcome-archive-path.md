---
author: AI Cockpit maintainers
title: "WI-148 — Archived Outcome path projection"
description: "Keep generated Outcome and human-handoff references valid after Work Item archive."
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-148-outcome-archive-path
---

# WI-148 — Archived Outcome path projection

The active Work Item directory is temporary lifecycle state. When a Work Item
is archived, generated Outcome, Task Outcome report, event, and `changedPaths`
references are projected to the corresponding archive paths before the archive
manifest and digests are written. This keeps raw records and human-facing
handoffs from pointing at active files that no longer exist.

The projection applies only while creating a new archive. Existing historical
archive bytes are immutable and are not backfilled or rewritten.

Evidence: `.ai/evidence/WI-148-outcome-archive-path.verification.json`.
Close decision: `.ai/decisions/WI-148-outcome-archive-path.close.json`.
