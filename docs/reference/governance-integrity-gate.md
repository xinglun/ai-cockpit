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

## Finalization head binding

For `feature_branch` and `pull_request` phases, a pre-merge finalization
receipt is valid only when its branch, pull-request, and worktree head resolve
to the reviewed checkout head. A later checkout is accepted only for a bounded
append of the canonical finalization transition or explicitly allow-listed
same-Work-Item governance records. The pending parity registry is one explicit
repository-level governance append used to keep a closed Work Item visible
while its three-language parity row is being completed. Code, tests, unrelated
evidence, or other repository changes force a fresh receipt and fail closed.

The gate does not choose verification tier or assurance. Risk/stage/policy
selection and reference-source file-by-file conformance are separate
verification boundaries and must not be inferred from this inventory.
