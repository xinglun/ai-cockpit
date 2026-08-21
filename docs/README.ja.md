---
author: AI Cockpit maintainers
title: "AI Cockpit ドキュメント"
description: "AI Cockpit を理解・採用・運用するための reader-first documentation home。"
audience:
  - adopter
  - contributor
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - documentation_architecture
---

# AI Cockpit ドキュメント

[English](README.md) | [中文](README.zh-CN.md)

ここは AI Cockpit の reader-first 入口です。まず目的に合う入口を選び、その後で
machine-facing contract を定義する technical reference に進んでください。

## ここから開始

- [設計思想](philosophy.ja.md) — なぜ evidence と human decision を明示するのか。
- [アーキテクチャ](architecture.ja.md) — runtime flow、ownership、boundary。
- [機能一覧と境界](capabilities.ja.md) — command、lifecycle、MCP、recovery。
- [Release と配布](release/distribution.ja.md) — install と release truth。
- [30 秒 command guide](capabilities.ja.md#機能一覧) — 現在の機能 index。

## Reader goal から選ぶ

| Goal | Start here | 到達点 |
| --- | --- | --- |
| Project を理解する | [設計思想](philosophy.ja.md) → [アーキテクチャ](architecture.ja.md) | evidence flow と product boundary を説明できる。 |
| 採用を判断する | [機能一覧](capabilities.ja.md) → [Installation](release/distribution.ja.md) | installation path と、変更されない範囲を理解する。 |
| Governed task を開始する | [機能一覧](capabilities.ja.md#governed-work-item-を実行する) → [Work Item ルール](work-items/README.ja.md) | inspect、attach、preflight、verify、close を実行する。 |
| Governance skeleton を作る | [機能一覧](capabilities.ja.md#work-item-skeleton-を作る) → [Command reference](reference/commands.ja.md) | `not_ready` skeleton と不足している human input を確認する。 |
| MCP client を設定する | [機能一覧](capabilities.ja.md#mcp-を使う) → [MCP と repository attach](release/distribution.ja.md#mcp-と-repository-attach) | 明示的な repository binding で server を起動し、response を読む。 |
| 結果を review・recovery する | [機能一覧](capabilities.ja.md#stop-と-recovery) → [敵対的検証](security/adversarial-validation.ja.md) | decision を読み、evidence を保持し、停止理由を修復する。 |
| 保守・監査する | [アーキテクチャ](architecture.ja.md) → [Protocol v1](protocol/v1/specification.ja.md) | ownership、boundary、machine-facing contract を見つける。 |

## Technical reference

- [製品境界](architecture/product-boundary.ja.md)
- [Runtime topology](architecture/runtime-topology.ja.md)
- [Release distribution architecture](architecture/release-distribution.ja.md)
- [バージョニング](architecture/versioning.ja.md)
- [Repository Protocol v1](protocol/v1/specification.ja.md)
- [Protocol compatibility](protocol/v1/compatibility.ja.md)
- [パフォーマンス受入れ](../tests/performance/README.ja.md)
- [実測パフォーマンスベースライン](performance/baseline.ja.md)
- [敵対的検証](security/adversarial-validation.ja.md)
- [Reference](reference/README.ja.md) — command、configuration、recovery。
- [Reference source parity](reference/reference-parity.ja.md) — adopted scope、意図的な差分、次の Work Item。

## Maintainer、audit、current work

- [Bootstrap Work Item ルール](work-items/README.ja.md)
- [Work Item ロードマップ](work-items/WI-03.ja.md)
- [Accepted Work Item: WI-34](work-items/WI-34.ja.md)
- [First-public-release Work Item: WI-35](work-items/WI-35.ja.md)
- [Corrective Work Item: WI-36](work-items/WI-36.ja.md)
- [Governance scaffolding Work Item: WI-38](work-items/WI-38.ja.md)
- [WI-34 installable-release design](superpowers/specs/2026-08-21-installable-release-homebrew-distribution-design.md)
- [WI-35 first-public-release design（英語 canonical）](superpowers/specs/2026-08-21-first-public-release-homebrew-tap-bootstrap-design.md)
- [WI-35 implementation plan（英語 canonical）](superpowers/plans/2026-08-21-first-public-release-homebrew-tap-bootstrap.md)

## 現在の実装境界

WI-03 から WI-38 までに implementation/readiness history を記録しています。WI-36 は
local acceptance 済みですが、hosted Release、Homebrew、public installation evidence
は WI-35 が担当します。Rust runtime が自分自身を governance できるまでは、
`docs/work-items` の Markdown bootstrap ルールを使います。この repository に V1 を
install しません。
