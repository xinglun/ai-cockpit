# Verification route

AI Cockpit routes verification facts through an explicit stage, policy plan,
dependency/affected graph, execution, evidence receipt, and CI boundary.

## Stages

`task`, `pre_ci`, `pr`, `merge`, and `release` are typed stages. `pre_ci` is
local feedback; `pr`, `merge`, and `release` remain independent provider or
protected-gate stages. Unknown stages fail closed. A stage is not an
assurance level.

## Orthogonal dimensions

`VerificationTier` (`T0`–`T3`) describes verification strength. `EvidenceAssurance`
(`SelfDeclared`, `RepositoryVerified`, `ProviderVerified`,
`EnterpriseVerified`) describes provenance. T3 never implies provider or
enterprise assurance; that assurance must be supplied by the actual provider
or external evidence.

The planner may propose a tier, but the requirement must be traceable to
Organization Policy, Project Policy, Release Policy, or a Protected Gate.

## Route and receipts

The route preserves `DependencyConfidence` (`complete`, `partial`, `unknown`)
and affected/protected-node facts. Cost and reuse are advisory and cannot lower
the selected requirement. A `VerificationPlanReceipt` records stage, initial
and final tier, independent assurance, reasons, escalations, and execution
facts. A Work Item route additionally binds `workItemId`, `repositoryId`, the
repository snapshot digest, `baseRevision`, policy references, required tier
and assurance, affected paths, and dependency confidence. Lifecycle validation
recomputes the declared policy requirement and rejects a missing, stale, or
tampered binding. A `pr`, `merge`, or `release` route must have a valid base
revision at the execution boundary; `task` remains base-revision independent.

When a Contract is policy-routed, `resolve_verification_route` binds declared
intent, scenario names, required scenarios, operation, and stage before the
command executes. Missing intent, required scenario coverage, or an
operation/stage mismatch is fail-closed. The same route feeds Agent Risk
validation, so typed required checks, `agentCapability`, and
`executionDecision` do not form a second CLI-only policy.

When an effective policy declares `T3` or `ProviderVerified`, a local Runtime
route cannot claim that requirement: verification stops before completion
evidence is written. Hosted/provider evidence must come from the actual
provider. A repository with no typed verification requirement keeps the
historical no-policy route and its legacy receipt compatibility.

Physical execution may be shared, but each Work Item receives its own bound
evidence receipt. No Work Item may use another Work Item's receipt as its own
authorization evidence.

## CI boundary

`pre_ci` is not hosted CI evidence. During the CI shadow phase, Runtime
verification and existing Cargo checks both run. A CI result cannot override a
red governance decision. Removing duplicate CI checks requires a later,
explicit convergence phase.

Cost observations are advisory telemetry only. They never turn unknown or
partial evidence into green governance.
