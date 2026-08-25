---
author: AI Cockpit maintainers
title: "WI-273 — reference inventory rebaseline"
workItemId: WI-273-reference-inventory-rebaseline
description: "Runtime の挙動を変更せず、file-level reference comparison ledger をレビュー済み default branch commit に再バインドします。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-273-reference-inventory-rebaseline
terminalArchive: .ai/work-items/archive/WI-273-reference-inventory-rebaseline.contract.json
terminalVerification: .ai/evidence/WI-273-reference-inventory-rebaseline.verification.json
terminalFinalization: .ai/decisions/WI-273-reference-inventory-rebaseline.finalize.json
terminalDecision: .ai/decisions/WI-273-reference-inventory-rebaseline.close.json
authority: canonical
---

# WI-273 — reference inventory rebaseline

## Intent

次の semantic comparison batch を開始する前に、file-by-file reference comparison ledger と
reader-facing documentation をレビュー済みの `origin/main` commit `487f019` に再バインドします。
これは metadata と documentation truth の更新であり、Runtime feature の変更ではありません。

## Scope

- inventory の target commit と tracked/working-tree metadata を更新します。
- WI-270/WI-272 の記録と4つの明示的な capability/profile migrate gap を含む既存分類を保持します。
- deferred path は deferred のままとし、metadata refresh で semantic work を close しません。
- English、Simplified Chinese、Japanese の comparison/parity documentation を同期します。
- historical `docs/work-items/**` と生成済み evidence は immutable のまま保持します。

## Boundary

Rust Runtime、CI workflow、Agent/MCP global configuration、reference source の挙動は変更しません。
deferred path を先に promote せず、archived Work Item evidence も書き換えません。Archive、
verification、decision の生成 record は installed Runtime が作成します。

## Acceptance

- inventory の target commit、tracked/working-tree count、path digest が clean な
  `origin/main` `487f01970c49e2b85d17b0cb0536f9d60c8f05e0` と一致します。
- ledger は 5,119 records：4,262 generated-history、163 implemented-different-by-design、
  1 implemented-equivalent、689 deferred-next-batch、4 migrate-gap です。
- generator と regression check は stale target revision を拒否し、現在の metadata を検証します。
- 3言語の comparison/parity documentation が同じ baseline と counts を使用します。
- documentation、inventory、governance、Contract が要求する quality check が通り、Runtime の
  business behavior は変わりません。

## Verification

- repository-bound call ごとに `--repo` を明示した installed Runtime。
- `bash tests/conformance/reference_file_inventory_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo . --report <report>`
- Contract が要求する full workspace quality と hosted checks。

## Terminal evidence

terminal path は Contract に従って installed Runtime が記録します：

- Archive: `.ai/work-items/archive/WI-273-reference-inventory-rebaseline.contract.json`
- Verification: `.ai/evidence/WI-273-reference-inventory-rebaseline.verification.json`
- Finalization: `.ai/decisions/WI-273-reference-inventory-rebaseline.finalize.json`
- Close: `.ai/decisions/WI-273-reference-inventory-rebaseline.close.json`
