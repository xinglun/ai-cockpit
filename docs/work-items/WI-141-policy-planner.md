---
author: AI Cockpit maintainers
workItemId: WI-141-policy-planner
title: Policy-driven verification planner
description: Make policy and stage the traceable source of verification requirements and reconcile historical artifact orphans.
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-141-policy-planner
---

# WI-141 — Policy-driven verification planner

This Work Item binds Planner requirements to explicit policy layers and
reconciles the two historical generated-artifact orphans found during audit.
It does not implement dependency confidence, cross-Work-Item execution reuse,
CI convergence, or performance targets.

Evidence is produced by the installed Runtime after the protocol and planner
tests, archive integrity tests, lint, and documentation acceptance pass.
