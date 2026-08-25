---
author: AI Cockpit maintainers
title: "WI-270 — Reference Contract semantics batch"
workItemId: WI-270-reference-contract-batch
description: "Compare the pinned reference source Contract and governance-semantics slice file by file."
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-270-reference-contract-batch
authority: canonical
---

# WI-270 — Reference Contract semantics batch

## Intent

This is the first semantic batch after the clean-up boundary. It compares the
pinned reference source one file at a time for Contract, intent, scenario,
acceptance, parallel, decision, and preflight behavior. The reference remains
a specification and behavior corpus; no reference Runtime or provider-global
configuration is copied into this repository.

## Scope

The initial slice is bounded to these reference surfaces and their Rust
counterpart documentation/inventory records:

- `docs/concepts/decision-states.*`
- `docs/features/work-item-parallelism.*`
- `docs/reference/safe-parallel-verification.md`
- `docs/reference/work-item-intelligence-interface.md`
- `docs/reference/work-item-state-machine.md`
- `docs/reference/work-item-status-interface.md`
- `scripts/ai_acceptance_policy.py`
- `scripts/ai_check_scenario_coverage.py`
- `scripts/ai_check_work_item.py`
- `scripts/ai_decision_protocol.py`
- `scripts/ai_intent_policy.py`
- `scripts/ai_parallel_verification.py`
- `scripts/ai_preflight_review.py`
- `scripts/ai_scenario_policy.py`
- `scripts/ai_work_item_state.py`
- `tests/test_acceptance_policy.py`
- `tests/test_ai_parallel_verification.py`
- `tests/test_checkpoint_intent.py`
- `tests/test_contract_and_policy.py`
- `tests/test_intent_policy.py`
- `tests/test_parallel_lifecycle_contract.py`
- `tests/test_preflight_review.py`
- `tests/test_scenario_coverage_gate.py`

The machine-readable inventory generator at
`tests/conformance/reference_file_inventory.py` is also in scope so the
classification cannot be lost when the ledger is regenerated.

Each path receives exactly one ledger classification, a Rust counterpart or a
recorded external boundary, an evidence reference, and an explicit gap or
deferral decision. A missing counterpart is not silently promoted to parity.

## Verification

- installed Runtime with explicit `--repo`
- reference inventory regression and governance integrity checks
- tri-language documentation acceptance
- targeted Rust tests for any bounded implementation correction
- visible human Outcome with status, unknowns, evidence, decision, and next action

## Boundary

This batch does not compare the remaining 720 deferred paths, implement a new
technology-stack adopter, or change user-global Agent/MCP configuration. If a
gap requires a Rust code change, the Contract is amended before editing and
the same Work Item owns the resulting evidence.
