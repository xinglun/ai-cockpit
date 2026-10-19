---
author: AI Cockpit maintainers
title: "WI-1089 — Blocked Outcome material review recovery"
description: "Bounded recovery after a failed Outcome and sensitive Contract amendment."
audience:
  - maintainer
  - reviewer
authority: explicit-user-authorization
workItemId: WI-1089-blocked-outcome-material-review-recovery
status: in_progress
lastVerifiedBy: WI-1089-blocked-outcome-material-review-recovery
---

# WI-1089 — Blocked Outcome material review recovery

A persisted failed Outcome remains red after a sensitive Contract amendment. A
fresh preflight and an identity-bound human `confirm_review` establish the
current Contract review; they do not accept material inspection risk. The
Runtime may offer `record_material_review_decision` only for a clean committed
Rust source with the single permitted, reviewable syntax Unknown, no Finding,
an exact current request and an opted-in Contract policy. The typed material
decision is independent and requires its own reviewer authority.

After a valid material receipt, the Runtime may offer one fresh verification
subject to its normal preconditions. The active failed Outcome remains red
until a real passing verification receipt replaces its current projection;
the append-only failure events remain readable. Finish and archive remain
blocked before that receipt. Dirty source,
unavailable material projection, Finding, contradictory evidence, stale or
foreign review input, and wrong recovery decisions remain rejected.

The retained canonical receipt is not a Missing case. Recovery validates its
bounded, nofollow read bytes and complete typed plan/execution/retention bindings
against a sealed predecessor Contract from the native amendment chain. Historical
snapshot and producer identity remain separate from current source and execution
authority. Unsupported or incomplete historical records fail closed with a
bounded diagnostic; generic Stale never grants this action.

The equivalent fixture retains a real old successful four-check receipt and a
real blocked Outcome. Acceptance requires exact current material review, zero
processes for plan-only, four actual passing checks with zero reuse and one
worker, fresh green preflight, then fixture finish and archive. Separate tests
preserve valid same-binding reuse and reject malformed or incomplete history;
a real failed check requires its own exact native retry. This is not real
Sentinel acceptance or authorization to finish/archive the actual Work Item.

The amended scope includes three repository implementation files (amendment,
evidence validation and status projection), repository/CLI tests and six
three-language WI/parity documents. Earlier six-node and combined-source results
remain historical evidence. The amended candidate needs its own complete checks
and exact independent material decision; unchanged finite consent is not approval
of new Rust blobs. PR1048 remains Draft; WI1090 stays paused.
