---
workItemId: WI-395-runtime-performance-extreme
title: Rust Runtime パフォーマンス最適化
author: AI Cockpit maintainers
description: ガバナンスを弱めず、重複する snapshot と Work Item status 処理を測定可能な形で削減する。
type: implementation
audience: [adopter, contributor, maintainer, reviewer]
authority: human-authorized
status: implemented
lastVerifiedBy: WI-395-runtime-performance-extreme
terminalArchive: .ai/work-items/archive/WI-395-runtime-performance-extreme.contract.json
terminalVerification: .ai/evidence/WI-395-runtime-performance-extreme.verification.json
terminalFinalization: .ai/decisions/WI-395-runtime-performance-extreme.finalize.json
terminalDecision: .ai/decisions/WI-395-runtime-performance-extreme.close.json
---

# WI-395 — Rust Runtime パフォーマンス最適化

[English](WI-395-runtime-performance-extreme.md) · [简体中文](WI-395-runtime-performance-extreme.zh-CN.md)

## Intent とインストール境界

request-scoped status、observe、Work Item 集約投影の Rust Runtime コストを測定し、
削減します。Runtime はマシン上で共有する外部 binary のままです。各 adopter は明示的な
`--repo` と独立した `.ai/` 状態を使い、参照源の installer、SDK/toolchain、Make/Python
runtime、V1 wire behavior はコピーしません。

## 有界な最適化

- Work Item 集約 status の同一 request 内で identity-bound snapshot を再利用する。
- 既存の Git index 読み取り中に source-tree digest を取得し、リモート既定メタデータを 1 回の限定 Git クエリで解決する。
- repository observation 中の再帰的な再ソートを避ける。
- 変更、unknown input、必須 check、evidence binding、fail-closed 判定を維持する。
- before/after の sample を記録し、provider/enterprise assurance と混同しない。

## 受入れ境界

性能目標は宣言した platform で identity-bound evidence とともに測定します。未達成なら
測定された gap を残し、verification を省略して解決したことにしません。Adopter は
installed/published Runtime で同じ cold/warm sequence を実行し、repository/Runtime identity
を分離して保持します。
