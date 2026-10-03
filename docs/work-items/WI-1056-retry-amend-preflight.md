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
