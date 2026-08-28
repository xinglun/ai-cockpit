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

A valid retry receipt whose archived Summary still records the repaired blocked
finish is consumed history, not a new terminal decision. Its pre-archive parity
ordering warning remains historical after merge and close; malformed or foreign
retry records still fail closed.

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

## Dynamic Work Item document projection

When a current Work Item declares `docs/reference/reference-parity*` or parity
registration in its Contract or active Summary, it is a parity/documentation
Work Item. The light gate then requires regular, non-symlink projections at
`docs/work-items/<WI>.md`, `docs/work-items/<WI>.ja.md`, and
`docs/work-items/<WI>.zh-CN.md` before verification, archive, or close. The
projection must have valid frontmatter and bind its `workItemId`; it does not
rewrite generated `.ai` history.

The same check applies to current-cycle archived parity Work Items. Missing,
malformed, foreign, or symlinked documents fail closed. Ordinary code Work
Items remain `active_non_parity` and are not forced to create documentation.
The policy is dynamic and is inherited by adopter repositories through their
repository-bound gate, so an adopter receives the same boundary without
copying the Rust Runtime.

The gate does not choose verification tier or assurance. Risk/stage/policy
selection and reference-source file-by-file conformance are separate
verification boundaries and must not be inferred from this inventory.

## Post-merge closure transition

Archive is intentionally created on the reviewed branch before the provider
merge. Finalization and the authoritative close receipt are recorded after
that merge, so the default branch can briefly contain an archived Work Item
without its close record. On a real GitHub `push` to the configured default
branch, the gate recognizes this state only when `HEAD` is an exact two-parent
merge commit and that merge adds the Work Item's archive Contract. It reports
`lifecycleState: awaiting_merge_close` rather than emitting a false
`missing_terminal_decision` finding.

This is a bounded transition, not a bypass or a permanent grace period. The
next non-transition default-branch check must observe the provider
finalization/close record; a direct commit, malformed merge, unrelated archive,
old unclosed Work Item, or missing/contradictory GitHub context remains
fail-closed. The workflow inherits GitHub's immutable `GITHUB_EVENT_NAME`,
`GITHUB_REF`, and `GITHUB_SHA` context; no branch-local or process-global
"current project" state is used.
