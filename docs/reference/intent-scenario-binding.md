---
author: AI Cockpit maintainers
title: Intent scenario and stage binding
description: Explain how human contract facts are bound to policy-derived verification routing.
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-143-intent-scenario-binding
---

# Intent scenario and stage binding

Contract intent and scenario coverage are human-owned facts. Before execution,
the route validator requires non-empty intent, every required scenario, and a
matching operation and stage. It then binds those facts to an already
policy-derived `VerificationRequirement`.

The validator does not read implementation prose to infer authority, risk,
assurance, or a T3 requirement. A high-risk route therefore still needs an
explicit policy rule and stage/gate references from the planner. Missing facts
or mismatched route bindings fail closed before verification starts.

`FinalDimensionsReceipt` remains the exact governed dimensions set. The
`fourPillarProjection` is presentation-only and cannot authorize or weaken a
route.
