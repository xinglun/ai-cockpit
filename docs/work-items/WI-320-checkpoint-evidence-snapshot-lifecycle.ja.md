---
author: AI Cockpit maintainers
title: "WI-320 — checkpoint evidence snapshot lifecycle"
workItemId: WI-320-checkpoint-evidence-snapshot-lifecycle
description: "編集前チェックポイントの履歴を許容しつつ、終端チェックポイントを現在の snapshot に固定します。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-320-checkpoint-evidence-snapshot-lifecycle
terminalArchive: .ai/work-items/archive/WI-320-checkpoint-evidence-snapshot-lifecycle.contract.json
terminalVerification: .ai/evidence/WI-320-checkpoint-evidence-snapshot-lifecycle.verification.json
terminalFinalization: .ai/decisions/WI-320-checkpoint-evidence-snapshot-lifecycle.finalize.json
terminalDecision: .ai/decisions/WI-320-checkpoint-evidence-snapshot-lifecycle.close.json
---

# WI-320 — checkpoint evidence snapshot lifecycle

## Intent と boundary

`before_edit` は実装前に記録する認可境界です。その後の編集と新しい
preflight は必ず新しい repository snapshot を作るため、その履歴を有効に
保ったまま `before_finish` 境界を弱めない必要があります。終端 checkpoint
は現在の Contract、repository、snapshot に bind し、宣言した verification
check は実際の結果に対応しなければなりません。

## Scope と acceptance

- identity、shape、stage、amendment chain が有効な過去の `before_edit` と
  amendment は保持できますが、現在の終端 evidence として扱いません。
- `before_finish` は現在の snapshot に bind し、stale、foreign、malformed、
  duplicate、symlink evidence は fail closed します。
- 必須 checkpoint check は決定的に導出し、存在しない verification 名を作りません。
- amendment、resume、lifecycle、repository isolation の回帰を維持します。
- 英語・簡体字中国語・日本語の文書でこの時間的な evidence 境界を説明し、
  最終 Runtime receipt にリンクします。

## Verification

checkpoint/lifecycle の targeted test、locked workspace test、documentation/parity
gate、review 済み branch の hosted check を実行します。repository-bound Runtime
command には常に明示的な repository path を渡します。

## Out of scope

Planner と parallel execution、performance、CI/release/adopter harness、global
Agent/MCP configuration、および大きな repository module の architecture split は
この限定的な修正の対象外です。

[English](WI-320-checkpoint-evidence-snapshot-lifecycle.md) ·
[简体中文](WI-320-checkpoint-evidence-snapshot-lifecycle.zh-CN.md)
