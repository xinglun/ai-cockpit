---
author: AI Cockpit maintainers
title: Governance cost metrics
description: Evidence-only execution-cost reporting for one repository-bound Work Item.
audience:
  - contributor
  - maintainer
  - adopter
status: implemented
authority: canonical
lastVerifiedBy: WI-345-reference-governance-cost-batch-15
---

# Governance cost metrics

The Rust Runtime exposes measured cost as advisory telemetry. Inspect one
repository and, optionally, one Work Item:

```sh
ai-cockpit diagnose --repo /path/to/repository
ai-cockpit diagnose --repo /path/to/repository --work-item WI-123
```

The JSON result is bound to the repository identity and reports snapshot Git
calls, files read/hashed, verification runs, executed and reused nodes,
elapsed time, bottleneck hints, evidence references, and explicit unknowns.
Verification receipts additionally expose typed `VerificationCostEstimate` and
`VerificationCostObservation` facts such as planned nodes, resource units,
parallelism, process counts, and execution time.

## Advisory boundary

Cost is not authority. An estimate or observation never changes
`VerificationTier`, `EvidenceAssurance`, policy requirements, protected nodes,
scope, or the final Outcome. Unknown worker/resource budgets, missing identity,
or invalid cached observations stay `unknown`/`partial`; they do not become a
green governance result. Physical execution reuse is separate from each
Work Item's identity-bound evidence receipt.

The reference project's JSONL phase/wait parser and source report wire shape
are not Rust protocol requirements. The Runtime intentionally does not invent
provider wait, human wait, token usage, P95, or lifecycle categories when the
repository does not supply them.

## Object-project inheritance

The same command and advisory boundary apply to every adopter repository. The
Runtime is shared, but the snapshot, Work Item, evidence, and cost facts are
request-scoped and repository-local. A cost report cannot authorize a change in
another repository or another Work Item.

