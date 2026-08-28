---
author: AI Cockpit maintainers
title: Governance complexity boundary
description: How the Rust Runtime observes repository growth without copying source maintenance tooling or rewriting audit history.
audience:
  - contributor
  - maintainer
  - adopter
status: reference
authority: canonical
lastVerifiedBy: WI-345-reference-governance-cost-batch-15
---

# Governance complexity boundary

The reference project has a Python/Make complexity report. The Rust Runtime
does not ship that source-specific scanner, its thresholds, or a global
complexity budget. This is intentional: a maintainer report is not a governance
decision and cannot be assumed to describe an adopter repository.

## What the Rust Runtime provides

Use an explicit repository context for the facts that are available:

```sh
ai-cockpit inspect --repo /path/to/repository
ai-cockpit status --repo /path/to/repository
ai-cockpit doctor --repo /path/to/repository
ai-cockpit diagnose --repo /path/to/repository --work-item WI-123
```

`inspect` reports the current snapshot and changed paths. `status` reports
repository compatibility and archive counts. `doctor` checks the attached
Runtime boundary. `diagnose` reports measured snapshot and verification cost
when a Work Item is selected; missing measurements remain `unknown`.

The repository CI integrity gate checks archive pairs, parity metadata, and
documentation consistency. These checks protect current repository facts; they
are not a replacement for the reference project's historical complexity
scanner and do not infer a complexity threshold.

## Archive and growth rules

Archived Contract, Summary, Outcome, evidence, and decision bytes are immutable
audit history. Growth alone does not authorize deletion, compaction, or a
change to another Work Item. Any proposed index repair or history compaction
must be a separate reviewed Work Item with an explicit retention decision.

Cost and performance observations are advisory. They cannot lower a required
verification tier, remove a protected check, or turn an unknown measurement
into a green Outcome. `VerificationTier` and `EvidenceAssurance` remain
independent dimensions.

## Object-project boundary

An adopter repository receives the same request-scoped rules through the shared
Runtime: every command carries `--repo`, and its archive/evidence state remains
local to that repository. The reference Python scanner, Make target, and source
threshold files are not silently installed into the adopter.

