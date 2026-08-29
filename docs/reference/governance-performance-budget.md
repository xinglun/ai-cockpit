---
author: AI Cockpit maintainers
title: Governance performance budgets
description: Identity-bound local performance measurements that never weaken required verification.
audience:
  - contributor
  - maintainer
  - adopter
status: implemented
authority: canonical
lastVerifiedBy: WI-345-reference-governance-cost-batch-15
---

# Governance performance budgets

Performance measurements are local engineering evidence, not permission to
omit a required check and not hosted-provider evidence. The Rust verification
crate defines typed `PerformanceBaseline`, `PerformanceSample`,
`PerformanceBudget`, and `PerformanceAssessment` records. A baseline requires
the Runtime version/digest, repository identity, capture time, samples, and
explicit maximum elapsed-time budgets.

The portable regression gate consumes captured baseline and candidate JSON:

```sh
tests/performance/regression_gate.sh baseline.json candidate.json
```

It rejects missing or zero-iteration samples, invalid identity, and budget
regressions. It does not build a source fallback and it does not change the
Contract's required verification graph. Verification command resource weights
and explicit resource budgets are likewise fail-closed before execution.

## Measurement boundary

The reference project's profile P95 report is not a Rust Runtime authority.
The target does not infer an established budget before enough samples, invent a
governance profile from timing, or present local timing as provider/enterprise
assurance. A measurement that is absent, stale, or identity-mismatched remains
unknown or fails closed.

Performance and governance strength are separate. `VerificationTier` and
`EvidenceAssurance` are not derived from elapsed time, cache hits, worker count,
or a budget result. Protected and policy-required nodes remain required even
when an over-budget report identifies a bottleneck.

## Dynamic verification selection

Detected Work Item commands use the same profile-authorized reuse path as
standalone auto-detected verification. The planner reuses a result only after
the repository, snapshot, profile, Runtime, command, scope, stage, runner,
base, toolchain, dependency, and policy identities match. A mismatch or
unknown impact executes the declared command and records the reason; it does
not silently widen reuse or downgrade a required check. Explicit custom
commands remain fresh so an operator must deliberately define any future
custom-command reuse contract.

## Rust-native optimization boundary

WI-395 removes redundant snapshot work from request-scoped status and
aggregate Work Item status projections, captures the source-tree digest while
the Git index is already being read, resolves remote default metadata in one
bounded Git query, and avoids re-walking directories just to sort intermediate
results. These optimizations preserve the same
snapshot, identity, evidence, and fail-closed decisions. They do not copy the
reference install flow: the Runtime remains one externally installed binary,
and each adopter binds it with an explicit `--repo` and its own `.ai/` state.

## Object-project inheritance

Adopter repositories can use the same identity-bound fixture and regression
gate, with their own repository and Runtime identities. The shared Runtime
does not store a global budget or current project, and one repository's timing
cannot authorize another repository's Work Item.

The same dynamic rule is inherited by adopter repositories after Runtime
upgrade: cold verification establishes the receipt, and an unchanged warm
repeat may reuse it only within that adopter's repository context. The adopter
acceptance receipt must record cold/warm elapsed time, executed/reused nodes,
selection reasons, Runtime identity, and repository identity.
