---
author: AI Cockpit maintainers
title: Cross-Work-Item physical execution reuse
description: Separate shared execution cost from Work Item authorization evidence.
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-144-cross-work-item-dedup
---

# Cross-Work-Item physical execution reuse

AI Cockpit may share the cost of one physical verification execution when all
physical identity facts match:

`repository + repository snapshot + command + environment + Runtime + toolchain`

The result is represented as `PhysicalExecution` and `ExecutionResult`. Neither
type contains a Work Item identity or grants authorization.

Each Work Item then creates its own `WorkItemEvidenceReceipt` from that result.
The Work Item id is part of the receipt digest, so Work Item A and Work Item B
have distinct receipts even when they share one physical result.

> No Work Item may reference another Work Item's Evidence Receipt as its own
> authorization evidence.

Physical reuse is an optimization only. It cannot lower a policy-required
VerificationTier, EvidenceAssurance, protected gate, authority requirement, or
freshness requirement. Any repository, snapshot, Runtime, command, or
toolchain mismatch uses a separate execution and fails closed when identity is
unknown.

Implementation evidence: `crates/cockpit-verification/src/lib.rs` and
`crates/cockpit-verification/tests/physical_execution.rs`.
