---
author: AI Cockpit maintainers
title: "WI-483 — WI-482 終端ドキュメント昇格"
description: "不変の証拠を書き換えず、WI-482 の終端ドキュメント投影を昇格する。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-483-wi482-doc-promotion
status: implemented
authority: canonical
lastVerifiedBy: WI-483-wi482-doc-promotion
terminalArchive: .ai/work-items/archive/WI-483-wi482-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-483-wi482-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-483-wi482-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-483-wi482-doc-promotion.close.json
---

# WI-483 — WI-482 終端ドキュメント昇格

この Work Item は、検証・クローズ済みの WI-482 ライフサイクルを三言語の
Work Item と reference-parity 投影へ昇格する。不変の Runtime 証拠、アーカイブ記録、
reference inventory は変更しない。

[English](WI-483-wi482-doc-promotion.md) · [简体中文](WI-483-wi482-doc-promotion.zh-CN.md)

## スコープ

- リポジトリのヘルパーで WI-482 の 6 つのドキュメント投影を昇格する。
- 正確な終端記録に結び付け、決定的な昇格を維持する。
- アーカイブ前に、この Work Item 自身のページと parity 行を登録する。

## スコープ外

Runtime/Core 実装、リリースまたは adopter 成果物、これらの投影を超える reference 実装 parity、
不変のガバナンスバイト。

## 受け入れ条件

1. WI-482 の 6 つの投影に証拠に基づく終端メタデータがある。
2. この Work Item に三言語ドキュメントとアーカイブ前 parity 行がある。
3. クローズ後に `promote_closed_work_item.py --check-all` が成功する。

## 検証

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `git diff --check`
