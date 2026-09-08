---
author: AI Cockpit maintainers
title: "WI-681 — P1 協作不変量カバレッジ対応付け"
description: "十の協作言語意味論的不変量を既存の自動テストカバレッジへ対応付け、正確なテストを引用し、残る欠落を境界の明確な後続 Work Item として示す。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-681-p1-invariant-coverage-mapping
status: in_progress
authority: authorized
lastVerifiedBy: WI-681-p1-invariant-coverage-mapping
---

[English](WI-681-p1-invariant-coverage-mapping.md) · [简体中文](WI-681-p1-invariant-coverage-mapping.zh-CN.md)

# WI-681 — P1 協作不変量カバレッジ対応付け

## 意図

AI Cockpit 協作言語専項の P1 範囲を推進する:WI-679 の協作言語契約で述べた
十の意味論的不変量それぞれについて、本リポジトリ自身のテストスイートに
今日既にこれを強制する自動テストが存在するかを答え、正確なテストを引用す
る。これは「まず再利用」の調査であり、テスト基盤の重複構築を避け、まだ
自動的な突合が存在しない不変量を正直に示すことを目的とし、リポジトリ所有
者からの明示的な委任に基づき、Work Item プロセスを通じて本専項を推進する。

## 境界

これは文書のみの Work Item である。
`docs/reference/collaboration-invariant-coverage.md`(+ zh-CN/ja)、
`docs/reference/README.md`(+ zh-CN/ja)への索引エントリ1件、本記録ページ、
および自身の reference-parity 登録を追加する。テストファイル、CLI/MCP の
挙動、Contract/Outcome スキーマはいずれも変更しない。発見された四件の欠落
テストの実装は明示的に対象外であり、納品文書内で境界の明確な後続 Work
Item として列挙し、それぞれ新しいパターンを発明するのではなく拡張可能な
既存のテストパターンを引用している。

## 受け入れとライフサイクル

- 「はい」/「部分的」と判定した各行は、納品時点で現在のテストソースを直接
  読んで確認したテストファイルと関数を引用している。
- 各欠落(「自動テスト未発見」)は正確に述べられ、再利用に基づく具体的な
  後続アプローチが示されている。
- `start → preflight → checkpoint → verify → finish → archive → close` が統治
  された経路であり、`user_visible_benefit_not_declared` は明示されたままで
  ある。
- 精査対象の head 上で `bash tests/docs/documentation_acceptance.sh` が合格
  する。

## 証拠

- archive: `.ai/work-items/archive/WI-681-p1-invariant-coverage-mapping.contract.json`
- verification: `.ai/evidence/WI-681-p1-invariant-coverage-mapping.verification.json`
- finalization: `.ai/decisions/WI-681-p1-invariant-coverage-mapping.finalize.json`(合併後に生成予定)
- close: `.ai/decisions/WI-681-p1-invariant-coverage-mapping.close.json`(合併後に生成予定)
