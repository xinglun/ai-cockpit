---
author: AI Cockpit maintainers
title: "WI-679 — P0 協作言語契約"
description: "七つの人-Agent 交流節目を既存の Runtime 事実へ対応付ける文書のみの索引に、十の検査可能な意味論的不変量を添えたもの。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-679-p0-collaboration-language-contract
status: in_progress
authority: authorized
lastVerifiedBy: WI-679-p0-collaboration-language-contract
---

[English](WI-679-p0-collaboration-language-contract.md) · [简体中文](WI-679-p0-collaboration-language-contract.zh-CN.md)

# WI-679 — P0 協作言語契約

## 意図

維持管理者にその場で説明を求められない新規ユーザーや貢献者のために、
リポジトリの実際の交流節目(タスク開始/範囲確認、授権請求、検証、阻断/復旧、
合併確認、Outcome、Agent/セッションの引き継ぎ)を既存の Runtime の事実へ対応
付ける横断的な索引を提供する。リポジトリ所有者からの明示的な委任に基づき、
Work Item プロセスを通じて AI Cockpit 協作言語専項を推進する。

## 境界

これは文書のみの Work Item である。
`docs/reference/collaboration-language-contract.md`(+ zh-CN/ja)と
`docs/reference/README.md`(+ zh-CN/ja)への索引エントリ1件を追加する。
CLI、MCP、Contract スキーマ、Outcome スキーマ、ライフサイクルの挙動はいずれ
も変更せず、既存の Outcome マーカー、決定状態の色、Receipt/授権再利用の規則、
阻断/復旧の用語の意味も変更しない——それらを既に定義済みの出典
(`.ai/glossary.md`、`docs/protocol/v1/specification.md`、
`docs/reference/outcome-report.md`、
`docs/reference/how-to-read-cockpit-status.md`、
`docs/reference/agent-workflow.md`、`docs/reference/troubleshooting.md`)
を引用・索引するのみである。指示により、外部参加者の募集・招待・面談・評価
は明示的に対象外とする。状態/遷移のシナリオ行列、自動化された不変量チェッ
ク、エンドツーエンドの一致性検証、引き継ぎ完全性チェックは、納品文書内で
後続の Work Item として明記されており、本 Work Item で提供済みとは主張しな
い。

## 受け入れとライフサイクル

- 納品された文書は七つの交流節目を網羅し、十の意味論的不変量を述べ、それぞ
  れが今日既に成立している根拠を示し、自動化チェックが未整備な箇所を明示す
  る。
- `bash tests/docs/documentation_acceptance.sh` が新規・変更ファイルに対して
  合格する。
- `start → preflight → checkpoint → verify → finish → archive → close` が統治
  された経路である。

## 証拠

- archive: `.ai/work-items/archive/WI-679-p0-collaboration-language-contract.contract.json`
- verification: `.ai/evidence/WI-679-p0-collaboration-language-contract.verification.json`
- finalization: `.ai/decisions/WI-679-p0-collaboration-language-contract.finalize.json`(合併後に生成予定)
- close: `.ai/decisions/WI-679-p0-collaboration-language-contract.close.json`(合併後に生成予定)
