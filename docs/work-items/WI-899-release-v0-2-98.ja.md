---
author: AI Cockpit maintainers
workItemId: WI-899-release-v0-2-98
title: Outcome、HCI、四方向、Issue #851 完了後の最終 v0.2.98 リリース
description: 前提となるすべての Work Item と Issue が完了した後に、レビュー済み main を公開する。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: user:release-after-all-work-items
lastVerifiedBy: WI-899-release-v0-2-98
---

[English](WI-899-release-v0-2-98.md) · [简体中文](WI-899-release-v0-2-98.zh-CN.md)

# WI-899 — Outcome、HCI、四方向、Issue #851 完了後の最終 v0.2.98 リリース

## 意図と境界

このリリース Work Item は、Outcome 交付、HCI 修正、四方向の収束、Issue #851、
Rust/ツールチェーン更新が完了して main に統合された後の最終公開経路である。
リリース識別子を更新し、ソース checkout とは独立して公開成果物を検証する。

対象リポジトリ以外の object repository は範囲外である。本 Work Item は
Outcome、host 表示、性能、ライフサイクル、ガバナンスの新しい動作を追加しない。

## 受入れ

- workspace package、lock metadata、現在の release 文書、生成 archive が
  v0.2.98 を一貫して識別する。
- 注釈付き v0.2.98 tag は不変でレビュー済み source commit に結び付き、既存
  tag は変更しない。
- provider Release は安定状態で、manifest、checksum、SBOM、archive、source
  identity、workflow handoff に結び付く。
- ダウンロードした v0.2.98 成果物が checksum、隔離 fresh-install、v0.2.97
  upgrade acceptance を通過し、一時 root が正確に清掃される。
- 英語・簡体字中国語・日本語の release/reference projection が同期し、最終
  Outcome が事実、制限、次の行動を正しく示す。

## 検証計画

高価な release job の前に format、version、Contract 前提条件を確認する。その後、
レビュー済み PR と dispatch 専用 release workflow を使い、失敗した候補の証拠を
保持し、publication handoff を消費してダウンロード成果物の adopter acceptance を
実行する。正確な branch/worktree cleanup と post-release evidence を結び付けてから
close する。
