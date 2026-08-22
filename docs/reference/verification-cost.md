---
author: AI Cockpit maintainers
title: Verification cost observation
description: Explain the advisory cost estimates and execution observations used to optimize verification safely.
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-146-verification-cost-observation
---

# Verification cost observation

AI Cockpit exposes verification cost as an auditable, advisory projection. It
reports planned nodes, nodes executed, reusable nodes, resource units,
elapsed time, processes spawned, and observed parallelism. A cost projection
never changes `VerificationTier`, `EvidenceAssurance`, policy requirements,
protected gates, or the final governance result.

## Two independent dimensions

Verification strength and evidence assurance remain separate:

- `VerificationTier`: `T0`, `T1`, `T2`, or `T3`.
- `EvidenceAssurance`: `SelfDeclared`, `RepositoryVerified`,
  `ProviderVerified`, or `EnterpriseVerified`.

Fast execution is not stronger verification, and a high tier does not imply a
provider or enterprise assurance level. Policy and protected-gate references
remain the source of required verification; the cost observer only records
what was planned and what actually ran.

## Estimate and observation

`VerificationExecutionPlan::cost_estimate` reports an estimate before
execution. `VerificationReceipt::cost_observation` projects execution facts
afterward. Both include a schema version, explicit confidence, and an
`advisoryOnly` marker. When worker/resource budgets, execution state, or
repository/Runtime identity are unknown, confidence is `partial` or `unknown`;
unknown measurements never become a green governance result.

Reuse and affected-verification reductions are observable facts. They do not
authorize skipping a protected node or a node required by policy. Physical
execution reuse also remains separate from per-Work-Item evidence receipts;
each Work Item must receive its own identity-bound receipt.

## Single and parallel work

The observation supports both a single Work Item and independent parallel
nodes. `maxConcurrentProcesses` records observed concurrency; it is not a
promise or a performance target. Resource budgets and dependency readiness
still bound execution, and protected nodes remain executed even when a cost
estimate is incomplete.

The engineering order is:

> **Verification Truth before Verification Cost.**

First preserve policy, tier, assurance, scope, evidence identity, and
protected gates. Only then use cost facts to reduce unnecessary execution
work. No hard latency or throughput target is an assurance claim.
