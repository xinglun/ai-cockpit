---
author: AI Cockpit maintainers
title: "WI-543 — reference file comparison batch 37"
description: "安全な conformance ledger check と七つの source checker 比較。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
workItemId: WI-543-reference-ledger-check-safety
lastVerifiedBy: WI-543-reference-ledger-check-safety
terminalArchive: .ai/work-items/archive/WI-543-reference-ledger-check-safety.contract.json
terminalVerification: .ai/evidence/WI-543-reference-ledger-check-safety.verification.json
terminalFinalization: .ai/decisions/WI-543-reference-ledger-check-safety.finalize.json
terminalDecision: .ai/decisions/WI-543-reference-ledger-check-safety.close.json
---

# WI-543 — Reference file comparison batch 37

## 目的

Pinned source commit `fde3380f81fea5fd2e288f7a8849f737dc074060` の維持対象
checker module 7 file を一つずつ比較し、inventory checker の `--check` を
read-only にします。Reference は specification/behavior corpus であり、
Python、Make、YAML、provider、source JSON wire implementation は Rust Runtime
へ copy しません。

## File-level result

| Reference path | Classification | Rust boundary |
| --- | --- | --- |
| `scripts/ai_check_task_outcome.py` | `implemented-different-by-design` | Typed OutcomeV2/TaskOutcomeReport、append-only event、localized human handoff、archive binding が portable boundary を担当し、source report wire/lexical policy は copy しません。 |
| `scripts/ai_check_test_weakening.py` | `implemented-different-by-design` | Snapshot-based Rust signal と fail-closed unknown が weakening boundary を担当し、source threshold/report format は source/provider policy のままです。 |
| `scripts/ai_classify_operation_impact.py` | `implemented-different-by-design` | Operation-time policy と scope evaluation が明示的な impact fact を作り、intent を推測せず source report format も取り込みません。 |
| `scripts/ai_close_work_item.py` | `implemented-different-by-design` | Typed lifecycle/finalization/ready-on-base gate が close を管理し、provider PR 操作と source runner orchestration は外部です。 |
| `scripts/ai_common.py` | `implemented-different-by-design` | JSON/Git/scope/redaction は typed Core/Protocol/repository/conformance に分散し、monolithic helper は Runtime dependency ではありません。 |
| `scripts/ai_critical_domain_guards.py` | `implemented-different-by-design` | Typed operation/authority、prompt injection、evidence forgery control は fail-closed を維持し、lexical classification を authority に昇格させません。 |
| `scripts/ai_dependabot_intake.py` | `not-applicable` | Dependabot event identity と bot branch intake は provider 固有です。Generic delegated evidence と source binding は利用できます。 |

## Ledger safety

`reference_file_inventory.py --check` は strict read-only です。generation、
rebaseline、apply option を write 前に拒否するため、誤った組み合わせで
append-only retired history を fresh projection に置換できません。Regression
wrapper は拒否と manifest byte identity を検証します。

Historical/retired record は immutable record として検証し、現在の pinned
path set だけが新しい batch decision の対象です。Source rename/remove/rebaseline
で完了済み比較を再オープンしません。

## Adopter inheritance

すべての attached adopter は shared Runtime、明示的な `--repo` context、隔離
された Contract/evidence/knowledge、fail-closed lifecycle、human Outcome handoff
を継承します。Source checker、Dependabot/provider event、source policy value、
source JSON wire format は継承しません。

## Verification

- `python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --check --source-commit fde3380f81fea5fd2e288f7a8849f737dc074060 --target-commit cb8248fdf8ac8d965d8d8eb7b53760147bd13fcd`
- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --locked --workspace --all-targets --all-features -- --test-threads=1`
