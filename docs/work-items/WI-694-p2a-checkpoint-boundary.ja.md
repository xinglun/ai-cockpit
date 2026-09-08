---
author: AI Cockpit maintainers
title: “WI-694 — P2-A checkpoint 責任境界”
description: “一つの lifecycle use case 内で checkpoint の観測、ガバナンス検証、永続化を分離する。”
audience: [maintainer, reviewer]
workItemId: WI-694-p2a-checkpoint-boundary
status: implemented
authority: human:repository-owner
lastVerifiedBy: WI-694-p2a-checkpoint-boundary
terminalArchive: .ai/work-items/archive/WI-694-p2a-checkpoint-boundary.contract.json
terminalVerification: .ai/evidence/WI-694-p2a-checkpoint-boundary.verification.json
terminalFinalization: .ai/decisions/WI-694-p2a-checkpoint-boundary.finalize.json
terminalDecision: .ai/decisions/WI-694-p2a-checkpoint-boundary.close.json
---

[English](WI-694-p2a-checkpoint-boundary.md) · [简体中文](WI-694-p2a-checkpoint-boundary.zh-CN.md)

# WI-694 — P2-A checkpoint 責任境界

## 境界

旧 WI-655 branch は stale になったため、最新の remote `main` から successor を開始する。
変更対象は repository lifecycle の `checkpoint_work_item` とその focused verification
だけであり、公開関数、Summary/checkpoint evidence の serialization、エラー優先順位、
authorization semantics、file layout は保持する。

## 責任分離

- `checkpoint_observe` は active Contract を読み、Git snapshot を一度取得し、Contract と
  repository snapshot digest を導出する。
- `checkpoint_governance_checks` は現在の preflight binding を検証し、取得済み snapshot を
 既存の governance authority に渡す。永続化は行わない。
- `checkpoint_work_item` は lifecycle の順序を保持し、検証成功後だけ checkpoint evidence と
  Summary を書き込む。

これは一つの use case に限定した抽出である。Port/trait の追加、governance rule の複製、
edit 境界を越えた snapshot 有効期間の拡大、finish/archive/close semantics の変更は行わない。

## 検証

Lifecycle entry、order、concurrency の focused tests で既存の fail-closed behavior を確認する。
Close 前に workspace format、Clippy、workspace tests、Runtime evidence、hosted checks、
terminal lifecycle records を完了させる。

## 残存リスク

Governance decision は従来の authority のままで、repository declaration を読む可能性がある。
この WI が防ぐのは checkpoint use case 本体による Contract/snapshot/digest の再取得だけである。
広範な observation-context threading は別の P1-B boundary とする。
