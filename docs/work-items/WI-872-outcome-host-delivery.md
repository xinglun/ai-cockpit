---
author: AI Cockpit maintainers
workItemId: WI-872-outcome-host-delivery
title: "Archived Outcome host delivery"
description: "Complete full Outcome handoff and host delivery boundary for archived Work Items."
audience:
  - adopter
  - contributor
  - maintainer
status: in_progress
authority: canonical
lastVerifiedBy: WI-872-outcome-host-delivery
---

# WI-872 — Archived Outcome host delivery

This Work Item closes the remaining gap between a Runtime-prepared full
Outcome and a host assistant-message event. It keeps return-only CLI/MCP
handoff honest when no host send API is available.

## Acceptance boundary

- One validated full body supplies normal and JSON archive output.
- A host adapter reports actual per-message receipts; capabilities never imply
  acceptance or display.
- Interrupted delivery resumes only from identity-bound accepted progress.
- Return-only adapters state `full_handoff_only` and unknown host status.
- Controlled adapter events prove ordering, segmentation, consecutive Work
  Items, interruption retry, and zero-send rejection.

Publication remains outside this Work Item and occurs only after all required
Work Items and evidence are complete.
