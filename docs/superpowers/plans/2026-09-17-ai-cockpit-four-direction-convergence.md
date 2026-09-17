---
title: "AI Cockpit four-direction convergence"
status: implemented
workItem: WI-865-four-direction-convergence
---

# Plan

WI-865 converges four bounded surfaces from the `ef4625f9` remote default
base: performance evidence, Outcome assembly and rendering, the prepared
ordinary-task path, and the responsibility map. The Work Item Contract remains
the authority for scope and acceptance; this plan is an implementation map,
not a second governance record.

## Boundaries

- Keep one repository-bound observation for each Outcome handoff and keep the
  human renderer free of repository, Git, and authorization decisions.
- Make release-grade percentile collection fail closed below 100 valid warm
  samples while retaining smaller diagnostic samples as non-acceptance data.
- Measure Runtime operations and development-cycle stages separately; report
  unavailable counters explicitly rather than substituting zero.
- Make `start --prepare` the ordinary entry path and keep provider/release
  finalization details in the detailed workflow reference.
- Do not publish or perform release-level adoption checks in this Work Item.

## Implementation and verification order

1. Run format, static, Contract, scope, and targeted regression checks.
2. Add the percentile-floor, Outcome-observation, delivered-behavior,
   release-projection, and tri-language path regressions.
3. Extend the portable benchmark to Outcome and verification planning; collect
   paired baseline/candidate samples with the same external executable boundary,
   environment identity, and scenario.
4. Run the complete workspace and documentation gates, then use the Runtime
   `verify → finish → archive → close` lifecycle and preserve all generated
   receipts.
5. After reviewed merge, perform exact provider finalization and close the Work
   Item. A later, separately authorized release Work Item owns publication.

## Acceptance evidence

The source tests are the contract for the cheap checks: `runtime_benchmark_stats`
and `p0_regression_gate` cover the 99/100 sample boundary; repository Outcome
tests cover bounded assembly, release facts, and summary/full rendering; CLI
Outcome handoff tests cover lifecycle output; and the documentation semantic
and acceptance scripts cover the prepared path in English, Simplified Chinese,
and Japanese. Captured benchmark JSON remains the source for any performance
claim, including raw samples, p50/p95/p99, environment identity, counters,
invalidations, and overlap status.
