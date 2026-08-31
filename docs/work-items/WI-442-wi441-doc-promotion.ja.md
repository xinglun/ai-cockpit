---
author: AI Cockpit maintainers
title: "WI-442 — WI-441 parity ledger ドキュメント投影"
workItemId: WI-442-wi441-doc-promotion
description: "クローズ済み WI-441 の終端証跡を三言語の reference-parity ledger に投影する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-442-wi441-doc-promotion
terminalArchive: .ai/work-items/archive/WI-442-wi441-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-442-wi441-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-442-wi441-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-442-wi441-doc-promotion.close.json
---

# WI-442 — WI-441 parity ledger ドキュメント投影

この Work Item は WI-441 の不変な終端パスを英語・簡体字中国語・日本語の
reference-parity ledger に投影します。Runtime の動作や WI-441 の evidence bytes は変更しません。

[English](WI-442-wi441-doc-promotion.md) · [简体中文](WI-442-wi441-doc-promotion.zh-CN.md)

## 範囲

- 三つの `docs/reference/reference-parity.*.md` ledger を更新する。
- archive、verification、finalization、close のパスを明示する。
- ローカルのみの reference-source 境界を保持する。

## 検証境界

Runtime の検証コマンドは `cargo test --locked --workspace` です。
`python3 tests/docs/promote_closed_work_item.py --check-all` が current を返し、
governance-integrity gate が成功した場合のみ投影を完了とします。
