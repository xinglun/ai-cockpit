---
author: AI Cockpit maintainers
workItemId: WI-1056-retry-amend-preflight
title: Recovery-retry amendment preflight candidate
description: Preserve the bounded Runtime start refusal and the local candidate test evidence for pending amendment-review request generation.
audience: [maintainer, reviewer]
status: in-progress
authority: user-authorized-bounded-governance-repair-exception
lastVerifiedBy: focused-local-tests-only
---

[简体中文](WI-1056-retry-amend-preflight.zh-CN.md) · [日本語](WI-1056-retry-amend-preflight.ja.md)

# WI-1056 recovery-retry amendment preflight candidate

This is a bounded governance-repair exception, not a Runtime-admitted Work Item
or a verification receipt. The branch `codex/wi1056-retry-amend-preflight` starts
from `origin/main` commit `a5bd06bf2932072eaef1ce1c33edcd8068b9d39b`.
`work-item new` created a `not_ready` scaffold. The formal `start` command was
rejected before activation with
`archived_work_item_scope_conflict:WI-1042-contract-amendment-environment-drift`
and `archived_work_item_scope_conflict:WI-1043-amendment-review-admission-fix`.
Neither archived Work Item nor its evidence was changed. The user's earlier
bounded governance-repair exception, delegated through Raydot for this exact
case, permits candidate tests and code in this isolated branch; it does not
turn `not_ready` into admission, approval, completion, or release authority.

The defect is limited to a validated `recoveryRetryPending=true` receipt plus a
subsequent sensitive Contract amendment that resets preflight to `not_run`.
The existing recovery retry bypasses stale preflight bindings during
verification precondition evaluation, so the policy-review error appears
first. The rc2 status projection did not recognize that error as recoverable
by request-generating preflight. This candidate admits only that preflight;
the request remains pending until an identity-bound human decision, and
verification must not spawn a project process before then.

The new CLI regression failed before the fix because `safeActions` omitted
`run_preflight`. After the narrow status-projection change, that same test
passed and checked current repository, Work Item, Contract, and snapshot
bindings, no recorded human decision, unchanged retry-pending state, and no
verification-command marker. The existing non-retry amendment test and the
repository-library tests also passed locally. These are candidate test facts,
not Runtime-bound verification of this Work Item. Independent review and any
further governance lifecycle remain outstanding.

A separate real-use check used the candidate Runtime without replacing the
shared installation. In Task 9's WI-1049, one formal amendment preserved the
existing A1 `development_cycle_cost` paths and added only the two A2
`runtime_benchmark_scenarios` Rust owner/test paths and their focused acceptance
check; the B `runtime_benchmark_stats` paths remained outside that batch. One
subsequent formal preflight returned `needs_human_confirmation` for Contract
`sha256:7de2a941e89fbeb2cef36f5a9c016a580f76ea71420a55aaafcdcba6572866df`
and repository snapshot
`sha256:fe207ccca471f47df7b27a61adf09901e9df622100c68d45730fc941dbd86cdd`.
The retry remained pending, no human decision was recorded, and Task 9 source
work did not resume. The regression also rejects review evidence with a wrong
repository, pre-amendment Contract, or wrong snapshot without writing a decision.
This observation proves request generation, not human approval or WI-1056
admission.
