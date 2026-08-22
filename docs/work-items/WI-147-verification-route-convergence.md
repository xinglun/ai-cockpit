# WI-147 — Verification route convergence

## Goal

Connect Contract and Policy requirements to typed VerificationStage, planning,
dependency/affected facts, execution receipts, and the CI boundary without
allowing reuse or cost optimization to weaken governance.

## Design baseline

Verification semantics come first. `VerificationTier` and
`EvidenceAssurance` are independent; the planner may only propose policy-
traceable requirements. Dependency confidence may be partial, and physical
execution sharing never shares authorization receipts. CI remains in shadow
comparison during this phase.

## Acceptance boundary

Unknown stages and tier downgrades fail closed. Receipts record route facts and
runtime/repository identity. Cost observations are advisory, empty plans have
zero parallelism, and malformed identity is unknown. No phase removes the
existing Cargo checks or fabricates provider/enterprise assurance.

See [Verification route](../reference/verification-route.md) and the
[Chinese](WI-147-verification-route-convergence.zh-CN.md) and
[Japanese](WI-147-verification-route-convergence.ja.md) versions.
