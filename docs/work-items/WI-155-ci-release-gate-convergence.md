---
author: AI Cockpit maintainers
title: "WI-155 — CI/release gate convergence"
description: "Keep release test execution deterministic and define the Phase 1 Runtime shadow as an execution smoke."
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-155-ci-release-gate-convergence
workItemId: WI-155-ci-release-gate-convergence
---

# WI-155 — CI/release gate convergence

WI-155 aligns the release source-quality gate with CI's deterministic
package-by-package Cargo test strategy. Each package is run with
`--test-threads=1`, while the verifier's internal worker-cap coverage remains
available inside an individual test binary.

The Runtime shadow is documented and checked as a Phase 1 **execution smoke**:
it validates an immutable public binary executing one repository-bound
verification command. Its receipt explicitly excludes policy-route/planner
coverage, affected-graph completeness, cross-Work-Item physical execution,
and per-Work-Item evidence receipt coverage. This boundary does not remove or
replace the existing Cargo and release gates.

Evidence: `.ai/evidence/WI-155-ci-release-gate-convergence.verification.json`.
Decision: `.ai/decisions/WI-155-ci-release-gate-convergence.close.json`.
