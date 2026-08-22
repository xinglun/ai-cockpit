---
author: AI Cockpit maintainers
title: "WI-149 — Structured release adopter decisions"
description: "Bind post-release adopter acceptance to complete, repository-bound Human Decision receipts."
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-149-release-decision-acceptance
workItemId: WI-149-release-decision-acceptance
---

# WI-149 — Structured release adopter decisions

Post-release adopter and N-1 upgrade acceptance must close each governed Work
Item with a complete structured Human Decision. The harness supplies the actor,
authority source, reason, evidence reference, policy reference, decision time,
and resume condition to the immutable Release binary.

After close, the harness requires a regular, non-symlinked
`.ai/decisions/<work-item>.close.json`, validates its Work Item, closed state,
confirmed decision, and structured fields, then copies it into the acceptance
artifact. A binding record adds the adopter `repositoryId`, Work Item ID,
decision digest, and validation result. Missing or mismatched decision
evidence fails closed; it never changes published Release truth.

The static wrappers enforce the structured close, copy, and validation boundary.
The tri-language release distribution guides describe the same acceptance
contract. Runtime core and CLI semantics remain outside this Work Item.

Evidence: `.ai/evidence/WI-149-release-decision-acceptance.verification.json`.
Close decision: `.ai/decisions/WI-149-release-decision-acceptance.close.json`.
