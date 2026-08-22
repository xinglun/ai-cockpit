---
author: AI Cockpit maintainers
title: Policy-driven verification planner
description: Explain how policy and stage requirements become a traceable verification plan.
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-141-policy-planner
---

# Policy-driven verification planner

The planner consumes explicit Organization, Project, and Work Item policy
layers in that order. A policy rule may carry a `VerificationRequirement` with
independent `requiredTier` and `requiredAssurance` values. `T3` means stronger
verification is required; it does not mean ProviderVerified or
EnterpriseVerified evidence.

For every selected operation and stage, the planner requires:

- a matching rule in every supplied policy layer;
- a valid requirement with a reference to its source policy id;
- a stage reference matching the requested stage; and
- a protected-gate reference when a protected gate is requested.

Missing rules or references fail closed. Lower policy layers may add evidence
or strengthen tier/assurance, but cannot weaken a higher layer. Planner output
records source policy ids and escalation reasons so a required tier never
becomes hidden operation-name logic.

The planner only defines verification requirements. It does not create human
authority, provider assurance, dependency completeness, execution reuse, or
performance exemptions.

Historical generated approach artifacts for WI-139C and WI-139F remain byte
preserved and are now bound to their archive manifests; no active artifact
orphan is treated as current project state.
