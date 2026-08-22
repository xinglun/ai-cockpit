---
author: AI Cockpit maintainers
title: Affected verification and dependency confidence
description: Explain conservative verification planning when dependency knowledge is complete partial or unknown.
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-142-affected-verification
---

# Affected verification and dependency confidence

The verification graph records `DependencyConfidence` separately from policy
truth:

- `complete` computes changed nodes and known downstream dependents;
- `partial` keeps that deterministic affected set but escalates those nodes to
  the stronger candidate tier and exposes `dependency_graph_partial`; and
- `unknown` conservatively includes every graph node and exposes
  `dependency_graph_unknown`.

Unknown or unsafe node references fail closed. A partial graph is not treated
as complete, but it also does not force a full highest-tier rerun when the
known affected boundary is sufficient. This projection only reduces execution
cost; it cannot weaken a policy-required tier, protected gate, authority, or
evidence requirement.
