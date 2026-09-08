---
author: AI Cockpit maintainers
title: "WI-710 — P1 人間決定請求の完全性テスト"
description: "WI-681 が指摘した不変量9の欠落を埋める:既存の二つの preflight テストを拡張し、HumanDecisionRequest の全フィールドが非空であることを断言する。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-710-p1-human-decision-request-completeness
status: implemented
authority: authorized
lastVerifiedBy: WI-710-p1-human-decision-request-completeness
terminalArchive: .ai/work-items/archive/WI-710-p1-human-decision-request-completeness.contract.json
terminalVerification: .ai/evidence/WI-710-p1-human-decision-request-completeness.verification.json
terminalFinalization: .ai/decisions/WI-710-p1-human-decision-request-completeness.finalize.json
terminalDecision: .ai/decisions/WI-710-p1-human-decision-request-completeness.close.json
---

[English](WI-710-p1-human-decision-request-completeness.md) · [简体中文](WI-710-p1-human-decision-request-completeness.zh-CN.md)

# WI-710 — P1 人間決定請求の完全性テスト

## 意図

WI-681 のカバレッジ対応付け(`docs/reference/collaboration-invariant-coverage.md`、
マージ待ち)が指摘した不変量9の欠落を埋める:「人による決定を要するすべ
ての問いは、決定の対象、影響、復旧/再開条件を明示しなければならない」に
ついて、`HumanDecisionRequest` のすべてのフィールドを網羅する専用の断言
がこれまで存在しなかった。本 Work Item は
`crates/cockpit-repository/tests/contract_preflight.rs` 内で既に
`needs_human_confirmation` に到達する二つの既存テスト(空のスキャフォール
ドと、高リスクのシナリオカバレッジゲート)を拡張し、新しいテストファイル
や本番コードを追加するのではなく、共有の断言ヘルパーを一つだけ新設する。
リポジトリ所有者からの明示的な委任に基づき、AI Cockpit 協作言語専項を継続
する。

## 境界

これはテストのみの Work Item である。既存ファイル1件
`crates/cockpit-repository/tests/contract_preflight.rs` のみを変更し、
ヘルパー関数1件と呼び出し箇所2件を追加する。本番のソースファイルは一切
変更しない。

## 受け入れとライフサイクル

- 新しいヘルパーは `what_happened`、`why_it_matters`、`question`、
  `resume_condition`、`options`、`recommended_option`、
  `recommendation_reason` が非空であること、`recommended_option` が提示さ
  れた選択肢のいずれかを指すこと、各選択肢の `id`/`label`/`effect` が非空
  であることを断言する。
- このヘルパーは既存の二つの `needs_human_confirmation` テストから呼び出
  され、独立して発生する二つの実際のシナリオをカバーする。
- `cargo test --locked -p cockpit-repository --test contract_preflight`
  (7/7)、`cargo fmt --check`、`cargo clippy --tests -- -D warnings` がすべ
  て合格する。
- `start → preflight → checkpoint → verify → finish → archive → close` が統治
  された経路であり、`user_visible_benefit_not_declared` は明示されたまま
  である。

## 証拠

- archive: `.ai/work-items/archive/WI-710-p1-human-decision-request-completeness.contract.json`
- verification: `.ai/evidence/WI-710-p1-human-decision-request-completeness.verification.json`
- finalization: `.ai/decisions/WI-710-p1-human-decision-request-completeness.finalize.json`(合併後に生成予定)
- close: `.ai/decisions/WI-710-p1-human-decision-request-completeness.close.json`(合併後に生成予定)
