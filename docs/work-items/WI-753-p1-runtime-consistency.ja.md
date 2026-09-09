---
author: AI Cockpit maintainers
title: "WI-753 — P1 中断と再開を通した Runtime の一致性"
description: "制御されたテストリポジトリで、表示状態、選択肢、Runtime の挙動が一致することを中断と再開を含めて実行可能に検査する。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-753-p1-runtime-consistency
status: in_progress
authority: authorized
lastVerifiedBy: WI-753-p1-runtime-consistency
---

[English](WI-753-p1-runtime-consistency.md) · [简体中文](WI-753-p1-runtime-consistency.zh-CN.md)

# WI-753 — P1 中断と再開を通した Runtime の一致性

## 意図

協作言語専項の第4節を納品する。制御された一時リポジトリで、Runtime が表示
した状態と選択肢が、選ばれた選択肢および Runtime が実際に許可する遷移と一致
することを、中断された検証と安全な再試行を含めて証明する。

## 境界

本 Work Item は一件の CLI 統合テストと三言語の協作リファレンスを変更する。
本番挙動、Outcome schema、外部参加者、別途承認済みの P2 オンボーディング/貢献
トラックは範囲外である。模擬的な人による選択は明示的に `TEST DATA ONLY` と
し、一時 fixture の下だけに書き込む。本リポジトリの授権記録にはならない。

## 受入れ

- `crates/cockpit-cli/tests/collaboration_consistency.rs` は、提示された
  `confirm_review`、選択前の拒否、選択後の checkpoint、中断後に検証合格を
  発行しないこと、再開後の現在の証拠を断言する。
- 検証の投影が変わると以前の preflight receipt は無効になる。テストは現在の
  `human_decision_recorded` を断言する前に fixture 用の新しい決定を記録する。
- 場景行列はこの実観測チェックを SCN-025 として記録するが、全状態空間の網羅
  は主張しない。
- `cargo fmt --check`、対象テスト、`cargo clippy --tests -- -D warnings`、文書
  受入れチェックがすべて成功する。

## 証拠

- archive: `.ai/work-items/archive/WI-753-p1-runtime-consistency.contract.json`
- verification: `.ai/evidence/WI-753-p1-runtime-consistency.verification.json`
- finalization: `.ai/decisions/WI-753-p1-runtime-consistency.finalize.json`
- close: `.ai/decisions/WI-753-p1-runtime-consistency.close.json`
