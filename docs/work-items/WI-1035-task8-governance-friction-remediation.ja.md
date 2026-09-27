---
author: AI Cockpit maintainers
workItemId: WI-1035-task8-governance-friction-remediation
title: Task 8 ガバナンス摩擦の修正
description: 単一の直列 Work Item で Task 8 の20項目すべてを是正し、Agent が実際に利用できる協調機能を公開し、リリース審査前に動作を検証する。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1035-task8-governance-friction-remediation
---

[English](WI-1035-task8-governance-friction-remediation.md) · [简体中文](WI-1035-task8-governance-friction-remediation.zh-CN.md)

# WI-1035 — Task 8 ガバナンス摩擦の修正

本 Work Item は WI-1033 handoff を引き継ぎ、20件すべてのガバナンス摩擦を単一の実装系列で扱う。内訳は元の handoff にある17件、独立して再現した close projection と引き継ぎ連絡の2件、および汎用的な検証証拠再利用プロセスである。権威ある受入基準、一覧、実施段階、証拠計画は Runtime に bind された Contract、および[Task 8 仕様](WI-1034-task8-governance-friction-remediation/spec.md)と[実施計画](WI-1034-task8-governance-friction-remediation/implementation-plan.md)に従う。

## 境界

- 実装は単一の Work Item、branch、worktree、repository context で直列に行い、巨大な単一コミットではなくレビュー可能な複数コミットに分ける。
- Agent 協調機能は発見可能で、実際に使用できなければならない。公開直後に manifest と MCP schema を確認し、実プロセスと複数 linked worktree による協調、および単一 WI の直列 fallback を受け入れ試験する。
- 宣言された検証を開始する前に Runtime の freshness と既存 receipt の binding を確認する。完全かつ fresh な証拠は再利用し、不足・無効な check だけを再実行し、正確な PR-head の hosted 証拠と local receipt を区別する。
- 検証、PR review、merge、cleanup、Work Item lifecycle はそれぞれ別の境界である。本タスクはリリース前で停止し、tag 作成や version 公開は行わない。
- 現在の状態は進行中。この projection は受入項目や検証結果の完了を主張しない。

## 受入概要

20件すべての Contract 基準を、最新のテストまたは実受入証拠、canonical CI gate、Runtime に bind された証拠へ関連付ける。証拠不足、候補 identity の stale、必須シナリオ未検証は blocker のままとする。

Hosted 検証は gate manifest に登録された共通 runner を使用する。候補 Runtime の現在の action admission に従って必要な場合のみ preflight を更新し、verification の直前に再度 admission を確認する。workspace coverage は同じ実行ファイルに bind された正式 receipt から生成し、package checks を重複実行しない。

`verify` コマンドの stdout は実行サマリーにすぎない。共通 runner はこれを分離して保存し、Runtime が `.ai/evidence/<WI>.verification.json` に書き込んだ正式 envelope を元のバイト列のまま Hosted receipt としてコピーする。Coverage は二層の receipt identity と Cargo workspace の全 package/node 集合を検証し、repository evidence との完全なバイト一致を必須とする。不正・不一致・再構成された receipt は fail closed にする。

同一バージョンの候補 Runtime を再ビルドし、旧 receipt の repository、Contract、version、構造と digest の検証が有効で executable digest だけが変わった場合、旧 receipt は stale となる。再利用や冗長な retry decision は行わず、共通 runner は候補 Runtime が許可した次の action に従う。Runtime version の変更や identity/evidence の不正は、従来どおり明示的な fail-closed recovery 境界を維持する。
