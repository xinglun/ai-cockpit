---
author: AI Cockpit maintainers
title: "Reference"
description: "User-facing command, configuration, and recovery references."
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - reference_index
---

# Reference

Use these pages after the [current reader route](../current/README.md) and first
capability walkthrough. The route indexes keep the ordinary user journey
separate from exact machine-facing details:

- [Getting started](../getting-started/README.md) — install and first attachment.
- [Features](../features/README.md) — capability goals and boundaries.
- [Operations](../operations/README.md) — lifecycle, recovery, upgrades, and acceptance.

- [Command reference](commands.md) — command groups, required bindings, and output behavior.
- [Configuration reference](configuration.md) — `.ai/cockpit.toml`, profiles, and generated records.
- [Troubleshooting and recovery](troubleshooting.md) — stop states and the next safe action.
- [Human-facing Outcome](outcome-report.md) — the readable result, risks, evidence, and next action.
- [Agent workflow and review boundaries](agent-workflow.md) — inherited Work Item, Outcome, release, and safety rules.
- [Verification route](verification-route.md) — typed stages, orthogonal tier/assurance, planning, receipts, and CI boundary.
- [Final replacement acceptance](final-replacement-acceptance.md) — the reproducible conformance and no-copy boundary.
- [Repository Protocol v1](../protocol/v1/specification.md) — normative storage and receipt contract.

The [reference parity record](reference-parity.md) is a maintainer/reviewer
comparison. It uses explicit truth states and is not a replacement for the
adopter route or a license to copy implementation history.
