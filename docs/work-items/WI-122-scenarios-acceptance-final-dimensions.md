---
author: AI Cockpit maintainers
title: "WI-122 — Scenario, acceptance, and final-dimension controls"
description: "Adds bounded validation and explicit recording for Contract/Summary governance projections."
audience:
  - adopter
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: work-item-acceptance
capabilityClaims:
  - scenario_coverage
  - acceptance_evidence
  - final_dimensions
---

# WI-122 — Scenario, acceptance, and final-dimension controls

WI-122 adds a read-only Contract/Summary validator and a bounded `controls`
writer. High-risk scenario coverage is fail-closed; legacy unnumbered
acceptance criteria remain compatible; numbered criteria use stable IDs and
per-item evidence. Intent alignment remains explicit when unknown.

Final acceptance receipts use the exact twenty reference dimensions. The
Runtime validates the receipt shape, identity, decision, and GO prerequisites;
it does not synthesize provider, enterprise, or adopter evidence. The optional
`fourPillarProjection` is a named presentation projection, not a `4D` field.

The implementation is exposed through `work-item validate`,
`work-item controls`, and the repository-bound MCP `work_item_validate` tool.
All commands require explicit repository binding.
