---
author: AI Cockpit maintainers
title: "WI-756 — P1 Runtime のみからの引き継ぎ再構築"
description: "新しい Agent またはセッションが会話履歴なしで協作引き継ぎを再構築できることを検査する、有界の制御リポジトリチェックを追加する。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-756-p1-handoff-reconstruction
status: recovered
authority: authorized
lastVerifiedBy: WI-756-p1-handoff-reconstruction-revalidation
terminalArchive: .ai/work-items/archive/WI-756-p1-handoff-reconstruction.contract.json
terminalVerification: .ai/evidence/WI-756-p1-handoff-reconstruction.verification.json
terminalFinalization: .ai/decisions/WI-756-p1-handoff-reconstruction.finalize.json
terminalDecision: .ai/decisions/WI-756-p1-handoff-reconstruction.close.json
---

[English](WI-756-p1-handoff-reconstruction.md) · [简体中文](WI-756-p1-handoff-reconstruction.zh-CN.md)

# WI-756 — P1 Runtime のみからの引き継ぎ再構築

## 意図

協作言語専項の第5節を納品する。制御された一時リポジトリで、新しいサブプロ
セスが会話履歴なしに、目標、範囲、完了状態、現在の証拠束縛、授権、永続化され
た阻断理由を再構築できることを証明する。

## 境界

本 Work Item は一件の CLI 統合テストと三言語の協作リファレンスを変更する。
本番挙動、Outcome schema、外部参加者の募集/インタビュー/評価、別途承認済みの
P2 オンボーディング/貢献トラックは範囲外である。テストは一時的な制御リポジ
トリの記録だけを読み、本リポジトリの授権記録は作成しない。

## 受入れ

- `crates/cockpit-cli/tests/collaboration_handoff.rs::new_agent_reconstructs_handoff_from_runtime_records_without_conversation_history`
  は新しいサブプロセス境界で Contract、Summary、status、永続化された active
  Outcome、現在の検証証拠を読み取る。
- テストは目標/範囲、完了済みと未完了、現在の証拠束縛と鮮度、authorized の
  authority、明示的な `finish.governance` の復旧条件、未宣言のユーザー向け便益
  を再構築し、完了や便益を推測しない。
- 場景行列はこの実観測チェックを SCN-026 として記録し、不変量カバレッジと
  契約ページは外部参加者の検証を前提としない有界の証拠として記述する。
- `cargo fmt --check`、対象テスト、`cargo clippy --tests -- -D warnings`、文書
  受入れチェックがすべて成功する。

## 証拠

- archive: `.ai/work-items/archive/WI-756-p1-handoff-reconstruction.contract.json`
- verification: `.ai/evidence/WI-756-p1-handoff-reconstruction.verification.json`
- finalization: `.ai/decisions/WI-756-p1-handoff-reconstruction.finalize.json`
- close: `.ai/decisions/WI-756-p1-handoff-reconstruction.close.json`
