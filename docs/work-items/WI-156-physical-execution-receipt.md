---
author: AI Cockpit maintainers
title: "WI-156 — Physical execution and Work Item evidence receipts"
description: "Keep shared physical computation separate from Work Item authorization and reject forged cost telemetry."
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-156-physical-execution-receipt
workItemId: WI-156-physical-execution-receipt
---

# WI-156 — Physical execution and Work Item evidence receipts

WI-156 keeps the physical execution result separate from the governance
receipt that authorizes a Work Item. A single physical result may be observed
by multiple Work Items, but each Work Item must bind and validate its own
receipt; no Work Item receipt can be reused as another Work Item's authority
or decision evidence.

Cost observations remain advisory telemetry. Persisted or cached observations
are accepted only when they exactly match the execution receipt, including
identity, counters, and canonical lowercase SHA-256 digests. A forged cache is
projected as `unknown` with `cost_observation_invalid`; it cannot make a
governance result green.

Evidence: `.ai/evidence/WI-156-physical-execution-receipt.verification.json`.
Decision: `.ai/decisions/WI-156-physical-execution-receipt.close.json`.
