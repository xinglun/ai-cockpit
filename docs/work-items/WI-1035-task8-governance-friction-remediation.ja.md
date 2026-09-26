---
author: AI Cockpit maintainers
workItemId: WI-1035-task8-governance-friction-remediation
title: Task 8 ガバナンス摩擦の修正
description: 単一の直列 Work Item で Task 8 の19項目すべてを是正し、Agent が実際に利用できる協調機能を公開し、リリース審査前に動作を検証する。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1035-task8-governance-friction-remediation
---

[English](WI-1035-task8-governance-friction-remediation.md) · [简体中文](WI-1035-task8-governance-friction-remediation.zh-CN.md)

# WI-1035 — Task 8 ガバナンス摩擦の修正

本 Work Item は WI-1033 handoff を引き継ぎ、19件すべてのガバナンス摩擦を単一の実装系列で扱う。内訳は元の handoff にある17件と、今回独立して再現した close projection と引き継ぎ連絡の2件である。権威ある受入基準、一覧、実施段階、証拠計画は Runtime に bind された Contract、および[Task 8 仕様](WI-1034-task8-governance-friction-remediation/spec.md)と[実施計画](WI-1034-task8-governance-friction-remediation/implementation-plan.md)に従う。

## 境界

- 実装は単一の Work Item、branch、worktree、repository context で直列に行い、巨大な単一コミットではなくレビュー可能な複数コミットに分ける。
- Agent 協調機能は発見可能で、実際に使用できなければならない。公開直後に manifest と MCP schema を確認し、実プロセスと複数 linked worktree による協調、および単一 WI の直列 fallback を受け入れ試験する。
- 検証、PR review、merge、cleanup、Work Item lifecycle はそれぞれ別の境界である。本タスクはリリース前で停止し、tag 作成や version 公開は行わない。
- 現在の状態は進行中。この projection は受入項目や検証結果の完了を主張しない。

## 受入概要

19件すべての Contract 基準を、最新のテストまたは実受入証拠、canonical CI gate、Runtime に bind された証拠へ関連付ける。証拠不足、候補 identity の stale、必須シナリオ未検証は blocker のままとする。
