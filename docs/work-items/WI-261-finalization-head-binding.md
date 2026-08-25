---
author: AI Cockpit maintainers
title: "WI-261 — Finalization head binding"
workItemId: WI-261-finalization-head-binding
description: "Reject stale pre-merge finalization receipts after non-governance drift."
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: documentation-acceptance
---

# WI-261 — Finalization head binding

## Intent

Bind pre-merge finalization evidence to the actual reviewed branch or
pull-request head. A receipt that merely repeats its own head fields must not
authorize a later checkout containing code that was never reviewed.

## Scope

- Resolve the reviewed head from a feature checkout (`HEAD`) or a synthetic
  pull-request merge checkout (the reviewed feature parent).
- Accept only an exact head match or an explicit append-only governance range
  for the same Work Item; reject later code and unrelated drift.
- Cover the binding and post-finalization drift with deterministic fixture and
  shell regressions.
- Document the rule in English, Chinese, and Japanese.

The fixture builder is included only to model a canonical finalization receipt
as an append-only commit; it does not change Runtime or Rust crates.

## Out of scope

Rust crates, provider configuration, global Agent/MCP configuration, and the
separate post-merge `stale_awaiting_merge_close` recovery lifecycle.

## Acceptance

1. A feature finalization receipt bound to an older checkout fails closed after
   a code commit.
2. A synthetic pull-request merge checkout binds to its reviewed feature
   parent.
3. Canonical/digest-suffixed finalization, same-Work-Item close, and fixed
   post-finalize evidence appends remain explicitly bounded.
4. Modified, deleted, renamed, unrelated, malformed, or non-governance paths
   are rejected.
5. Tri-language reference documentation states the same boundary.

## Verification

- `bash tests/ci/governance_integrity_gate_test.sh`
- `python3 -m py_compile tests/ci/governance_integrity_gate.py tests/ci/fixtures/governance-integrity/build_fixture.py`
- Contract-declared workspace verification after the focused gate passes.
