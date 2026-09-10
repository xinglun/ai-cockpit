---
author: AI Cockpit maintainers
title: "WI-773 — P1 status canonical fact reuse"
description: "Measure request-scoped reuse of canonical finalization receipt facts and decline the candidate when the pre-registered benefit is not demonstrated."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: authorized
workItemId: WI-773-p1-status-canonical-fact-reuse
lastVerifiedBy: WI-773-p1-status-canonical-fact-reuse
terminalArchive: .ai/work-items/archive/WI-773-p1-status-canonical-fact-reuse.contract.json
terminalVerification: .ai/evidence/WI-773-p1-status-canonical-fact-reuse.verification.json
terminalFinalization: .ai/decisions/WI-773-p1-status-canonical-fact-reuse.finalize.308053f58465bd212e3edb34f296a7628cb59422d0c3ccc2af112e3c75168201.json
terminalDecision: .ai/decisions/WI-773-p1-status-canonical-fact-reuse.close.json
---

# WI-773 — P1 status canonical fact reuse

## Contract and boundary

This Work Item started from remote `origin/main` at `5a24d4c0df865ece469822dbdc0dcc36eda07d85`.
The hypothesis was that `status` historical finalization projection rereads and reparses the
canonical `*.finalize.json` receipt after the outer inventory loop had already observed it.
The proposed reuse was limited to one immutable status observation. Cross-request, cross-repository,
pre/post-mutation reuse, governance-rule changes, and unrelated performance work were out of scope.

The Contract pre-registered acceptance at least 5% improvement for both warm independent-CLI
`status` p50 and p95, with governance outputs equivalent and non-target budgets respected.

## Measurement and decision

The paired evidence is:

- `tests/performance/fixtures/WI-773-status-canonical-fact-reuse-baseline.json`
- `tests/performance/fixtures/WI-773-status-canonical-fact-reuse-candidate.json`

Both records use the same detached clean fixture, repository identity, HEAD, 445 canonical
finalization receipts, and 651 archived Work Item contracts. The harness retains raw order,
first measurement, one independent CLI warmup, 20 warm samples, nearest-rank quantiles, scenario
facts, and explicit unavailable reasons. The fixture is classified as `many-historical-wi`.

| path | warm p50 | warm p95 | decision |
| --- | ---: | ---: | --- |
| baseline `status` | 1062.470 ms | 1095.271 ms | reference |
| candidate `status` | 1089.892 ms | 1723.202 ms | declined |

The candidate is about 2.58% slower at p50 and about 57.3% slower at p95 in this paired run.
It therefore does not meet the Contract threshold. The candidate production change was reverted;
no governance behavior, evidence semantics, repository isolation, authorization boundary, or
recovery behavior was changed.

The portable harness could not obtain a trustworthy filesystem type on this macOS host:
`stat -f %T` returned `/`, so comparison-key and filesystem comparability are explicitly
unavailable. Runtime phase timings, read bytes, hashed bytes, Git call counts, child-process
counts, peak memory, and cache invalidation reasons remain unavailable where the Runtime does
not expose reliable counters; no unavailable value was filled with zero. Resident MCP and
concurrent validation were not measured by this CLI harness.

## Correctness and verification

The candidate probe used a focused observation-local fact path and verified that the canonical
head read wrapper was not called a second time. The probe passed, but the implementation was
declined because the paired end-to-end result did not meet the threshold. The retained tree has
the original implementation and no candidate helper.

The benchmark gate was run with the retained raw records. It correctly failed closed on the
unavailable filesystem comparison and, in the second run, the candidate `status` p95 budget.
This is evidence of measurement limitations and candidate rejection, not a reason to weaken the
gate. The next performance Work Item should target the dominant measured stage only after a
fresh bottleneck profile.

## Outcome

Outcome: 🟡 candidate declined; measurement evidence retained. Issue count: 1 (insufficient
performance benefit and unstable tail). Blockers: no accepted production optimization from this
hypothesis. Resolved: duplicate-read hypothesis was isolated, measured, and fail-closed rejected.
Risk: the current status path remains unchanged and therefore retains its prior cost. Verification:
raw paired benchmark fixtures, P0 gate output, focused probe, and post-revert repository tests.
Next action: use the P0 scenario matrix to select the next independently scoped bottleneck; do
not revive this candidate without a new Contract and evidence.
