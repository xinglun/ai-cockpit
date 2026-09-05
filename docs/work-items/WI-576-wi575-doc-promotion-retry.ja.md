---
author: AI Cockpit maintainers
title: "WI-576 — WI-575 ドキュメント昇格 retry"
description: "検証可能な lifecycle 順序で WI-574 の終端ドキュメント投影を再配信する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-576-wi575-doc-promotion-retry
lastVerifiedBy: WI-576-wi575-doc-promotion-retry
terminalArchive: .ai/work-items/archive/WI-576-wi575-doc-promotion-retry.contract.json
terminalVerification: .ai/evidence/WI-576-wi575-doc-promotion-retry.verification.json
terminalFinalization: .ai/decisions/WI-576-wi575-doc-promotion-retry.finalize.ff9b14cb37866d6e475e2dfc72c705bd289d494ae54790b2b5625c5292a94d42.json
terminalDecision: .ai/decisions/WI-576-wi575-doc-promotion-retry.close.json
---

[English](WI-576-wi575-doc-promotion-retry.md) · [简体中文](WI-576-wi575-doc-promotion-retry.zh-CN.md)

# WI-576 — WI-575 ドキュメント昇格 retry

## 目的

PR #556 が証明できない lifecycle 順序のため immutable failed delivery として閉じられた
WI-575 の終端ドキュメント投影を再配信する。この successor は失敗を provider の監査履歴
として保持し、三言語 parity 行の登録、archive、review、merge、close、昇格という順序だけを
修正する。

## 範囲と境界

- WI-574 の Work Item ページと三つの reference-parity 行を昇格する。
- この successor の三言語ページと parity 登録を維持する。
- PR #556 の失敗を保持し、merge 済みと主張せず、WI-575 の bytes を書き換えない。

Runtime、対象 repository、global Agent/MCP 設定、reference source の実装コピー、release
公開、過去の governance bytes は対象外とする。

## 受入れ

1. WI-576 parity 登録は archive 前に commit し、verified close 前は `In progress` のままにする。
2. WI-574 ページは検証済み終端 evidence に基づく場合だけ Implemented にする。
3. 三つの parity ledger は close 後に正確な終端パスを含む。
4. ドキュメント、governance、status、workspace、diff checks が通る。
5. WI-575 その他の過去 governance record を書き換えない。

## 検証

- `tests/docs/documentation_acceptance.sh`
- `tests/docs/parity_status_check.sh`
- `tests/docs/pending_parity_registry_test.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `python3 tests/docs/work_item_status_consistency.py --repo .`
- `cargo test --locked --workspace`
- `git diff --check`
