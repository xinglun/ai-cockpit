---
author: AI Cockpit maintainers
title: "WI-270 — Reference Contract semantics batch"
workItemId: WI-270-reference-contract-batch
description: "固定した reference source の Contract と governance semantics を file-by-file で比較します。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-270-reference-contract-batch
authority: canonical
---

# WI-270 — Reference Contract semantics batch

## Intent

これは cleanup boundary 後の最初の semantic batch です。固定した reference source の
Contract、intent、scenario、acceptance、parallel、decision、preflight の動作を一つずつ比較します。
reference は仕様と behavior corpus であり、Runtime や provider-global configuration をコピーしません。

## Scope

最初の slice は次の reference surface と Rust 側の documentation/inventory record に限定します。

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

機械可読 ledger generator `tests/conformance/reference_file_inventory.py` も scope に含め、
ledger 再生成時にこの batch の分類が失われないようにします。

各 path は一つだけの ledger classification、Rust counterpart または external boundary、evidence、
gap/延期の判断を持ちます。counterpart がない場合に parity として扱いません。

## Verification

- 明示的な `--repo` を付けた installed Runtime
- reference inventory regression と governance integrity gate
- 三言語 documentation acceptance
- bounded な実装修正があれば対象 Rust test
- status、unknowns、evidence、decision、next action を含む visible human Outcome

## Boundary

残りの 720 deferred path、新しい technology-stack adopter、user-global Agent/MCP configuration は対象外です。
gap が Rust code 修正を要求する場合は、編集前に Contract を amend し、同じ WI で evidence を残します。
