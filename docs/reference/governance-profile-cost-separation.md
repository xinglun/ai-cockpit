---
author: AI Cockpit maintainers
title: Governance profile and cost separation
description: How profile intensity, verification strength, assurance, and operation-specific escalation remain distinct.
audience:
  - contributor
  - maintainer
  - adopter
status: implemented
authority: canonical
lastVerifiedBy: WI-345-reference-governance-cost-batch-15
---

# Governance profile and cost separation

The Rust Runtime has `light`, `standard`, and `strict` quality routes. A route
describes verification intensity for the change; it is not a cost target and
does not encode an organization's assurance level. `release` is an operation
class, not a fourth profile.

Keep these dimensions separate:

- `VerificationTier` (`T0`–`T3`) describes verification strength.
- `EvidenceAssurance` (`SelfDeclared`, `RepositoryVerified`,
  `ProviderVerified`, `EnterpriseVerified`) describes evidence provenance.
- Cost observations describe measured work and are advisory only.

The effective route is raised by stage, risk, declared operation, protected
gates, and repository policy. A planner may propose a tier or escalation, but
the requirement must remain traceable to policy or a protected gate. A
requested profile can raise the route; it cannot lower the effective floor.

For a release-related operation, the route may require release preflight and
distribution evidence. Non-release strict work does not inherit that release
graph merely because it is strict. If policy requires `T3` or
`ProviderVerified`, a local-only run cannot claim completion; the relevant
provider or external evidence must be supplied.

The same profile and cost boundary is used by an adopter repository through the
shared Runtime. No process-global current project or hidden planner policy is
created, and timing/caching never authorizes a weaker route.

