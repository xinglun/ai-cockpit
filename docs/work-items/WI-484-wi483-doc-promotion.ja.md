---
author: AI Cockpit maintainers
title: "WI-484 — WI-483 終端ドキュメント昇格"
description: "不変の証拠を書き換えず、WI-483 の終端ドキュメント投影を昇格する。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-484-wi483-doc-promotion
status: implemented
authority: canonical
lastVerifiedBy: WI-484-wi483-doc-promotion
terminalArchive: .ai/work-items/archive/WI-484-wi483-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-484-wi483-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-484-wi483-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-484-wi483-doc-promotion.close.json
---

# WI-484 — WI-483 終端ドキュメント昇格

この Work Item は、検証・クローズ済みの WI-483 ライフサイクルを三言語の
Work Item と reference-parity 投影へ昇格する。不変の Runtime 証拠、アーカイブ記録、
reference source の意味は変更しない。

[English](WI-484-wi483-doc-promotion.md) · [简体中文](WI-484-wi483-doc-promotion.zh-CN.md)

## スコープ

- リポジトリのヘルパーで WI-483 の 3 つのドキュメント投影を昇格する。
- 正確な終端記録に結び付け、決定的な昇格を維持する。
- アーカイブ前に、この Work Item 自身のページと parity 行を登録する。

## スコープ外

Runtime/Core 実装、リリースまたは adopter 成果物、新しい reference 比較パス、
不変のガバナンスバイト。

## 受け入れ条件

1. WI-483 の 3 つの投影に証拠に基づく終端メタデータがある。
2. 3 つの reference-parity 行が WI-483 を Implemented とし、同じ終端証拠へリンクする。
3. この Work Item に三言語ドキュメントとアーカイブ前 parity 行がある。
4. クローズ後に `promote_closed_work_item.py --check-all` が成功する。

## 検証

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `git diff --check`
