---
author: AI Cockpit maintainers
title: "WI-674 — P1 Repository 責務分割"
description: "公開挙動と永続化互換性を保ったまま cockpit-repository の内部責務を分割します。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-674-p1-repository-split
status: in_progress
authority: authorized
lastVerifiedBy: WI-674-p1-repository-split
---

[English](WI-674-p1-repository-split.md) · [简体中文](WI-674-p1-repository-split.zh-CN.md)

# WI-674 — P1 Repository 責務分割

## Intent

公開 API、wire format、永続化レイアウト、エラー挙動、認可 semantics、ライフサイクル
判定を変更せずに `cockpit-repository` の内部責務を分割します。これは構造的な
refactor であり、測定済みの性能または認知的 benefit を主張するものではありません。

## Boundary and dependency map

| Module | Responsibility | 主な依存と利用者 |
| --- | --- | --- |
| `lifecycle.rs` | Work Item entry、checkpoint、preflight、finish、verification、recovery、ライフサイクル境界 helper | `lib.rs` の共有 Repository type と governance/evidence helper、status/readiness projection、evidence store、Outcome projection helper |
| `evidence_store.rs` | 再利用可能 receipt の保存、validity binding、有界 read、atomic publication、capability filesystem helper | Protocol digest と Repository identity；lifecycle と execution-context が利用；保存 path と bytes は不変 |
| `execution_context.rs` | Repository/runtime execution context、executable identity、staging、shebang と environment identity、verification reuse assessment | Git snapshot と evidence-store binding；verification と lifecycle が利用；reuse rule は不変 |
| `status_projection.rs` | Repository status、readiness、worktree topology、historical debt、archive-close projection | Git snapshot と archive/history reader；lifecycle entry と status caller が利用；projection は read-only |
| `lib.rs` | 公開 API、共有 protocol type、安定した re-export、横断 helper | module API を再 export し、既存の serialization と persistence contract を保持 |

依存方向は保守的に保ちます。抽出 module は `lib.rs` の共有 Repository 定義を利用し、
`lib.rs` は既存の公開 surface のみを再 export し、既存の cross-module call には限定的な
internal import で対応します。新しい crate や framework は導入しません。

## Compatibility

base は remote default `origin/main` の
`c1f1f1d9f17ec2242a59f287a4368bd6bda5993e` です。既存 implementation の移動だけを行い、
protocol field は追加しません。公開 function、serialized artifact、error path、persistence
path、status semantics は bytes と挙動の互換性を維持する想定です。最終確認には下記の
workspace および hosted check が必要です。

## Verification evidence

module 移動後、lifecycle entry/order、recovery decision/events/revalidation、receipt store、
evidence assurance、repository context、verification context、verification service、status
projection の定向 behavior test 合計 107 件が pass しました。

未確認事項は、完全な `cargo test --locked --workspace`、format check、Clippy、documentation/
parity check、Runtime `verify`、hosted PR check、terminal lifecycle record です。move-only
refactor の過程で behavior defect は見つかっていません。発見した場合は別の governed
change として記録・処理し、この構造 WI に隠しません。

## Scope exclusions

Outcome P0-A/P0-B の挙動、P1-A cognitive-benefit evaluation、P2 first-use documentation、
future governance-principle WI、他 agent の worktree、user-global Agent/MCP configuration は
本 Work Item の範囲外です。
