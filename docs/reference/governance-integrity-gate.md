---
author: AI Cockpit maintainers
title: Governance integrity gate
description: "Fail-closed inventory of current Work Item records, evidence, terminal decisions, and documentation bindings."
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-198-governance-gate-default-branch-discovery
---

# Governance integrity gate

`tests/ci/governance_integrity_gate.py` discovers Work Items from the repository
records instead of maintaining a fixed ID list. It checks the current release
cycle, evidence identity, terminal decision, Outcome, and three-language parity
bindings. Findings are deterministic and fail closed.

## Recovery is not completion

A predecessor with a valid `.ai/decisions/<WI>.recovery.json` is reported as
`lifecycleState: recovered`. The receipt must bind the predecessor and a
non-empty successor, the repository identity, a reason, and evidence refs. A
recovered predecessor may retain a red/blocked immutable Outcome; that Outcome
is never promoted to green and is never treated as merge or release approval.

Missing, malformed, foreign, or weakly bound recovery receipts remain errors.
The successor must independently pass its own Contract, evidence, Outcome,
parity, and terminal-decision checks.

## Finalization binds the reviewed checkout

For a feature-branch or pull-request checkout, the finalization receipt is
valid only when its branch, pull-request, and worktree heads resolve to the
actual reviewed checkout (`HEAD`, or the reviewed feature parent of a
synthetic merge checkout). A receipt that is internally self-consistent but
points at an older commit is rejected; this prevents a later code commit from
silently inheriting an earlier finalization.

An ancestor receipt may cross the boundary only through a bounded,
append-only governance update for the same Work Item: canonical or digest-
suffixed finalization records, the repository-local close decision, and the
two fixed post-finalize evidence records. Any code or unrelated record,
modified/deleted/renamed path, or later non-governance drift remains
fail-closed. The receipt head is therefore a binding to the reviewed source,
not just a value copied into the receipt itself.

## Detached pull-request checkouts

Hosted pull-request jobs can run from a detached merge checkout without
`refs/remotes/origin/HEAD` or event base-branch metadata. In that case the gate
uses only the immutable Contract `resourceContext.baseBranch` as a narrow
default-branch fallback. The fallback is accepted only when the receipt and
Contract resource contexts match exactly; repository, PR URL/number, provider,
remote, branch, worktree, base/head revisions, runtime, evidence, and Contract
digest checks remain mandatory. If an external event or remote declares a
different base branch, the receipt is rejected. Missing or contradictory
identity remains fail-closed.

The gate does not choose verification tier or assurance. Risk/stage/policy
selection and reference-source file-by-file conformance are separate
verification boundaries and must not be inferred from this inventory.
