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

The current implementation scope is the repository status projection, its
repository/CLI/MCP regression tests, and the three WI-1089 documentation
pages. Reference-parity registration is a separate Contract scope decision.
