---
author: AI Cockpit maintainers
workItemId: WI-911-release-v0-2-99
title: Outcome 言語、HCI、四方向、Issue #851 完了後の最終 v0.2.99 リリース
description: 前提となるすべての Work Item と Issue が完了した後に、レビュー済み main を公開する。
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-911-release-v0-2-99
terminalArchive: .ai/work-items/archive/WI-911-release-v0-2-99.contract.json
terminalVerification: .ai/evidence/WI-911-release-v0-2-99.verification.json
terminalDecision: .ai/decisions/WI-911-release-v0-2-99.close.json
---

[English](WI-911-release-v0-2-99.md) · [简体中文](WI-911-release-v0-2-99.zh-CN.md)

# WI-911 — Outcome 言語、HCI、四方向、Issue #851 完了後の最終 v0.2.99 リリース

## 意図と境界

このリリース Work Item は、Outcome を会話言語で出力する修正、HCI 修正、四方向の
収束、Issue #851、Rust/ツールチェーン更新が完了して main に統合された後の最終公開
経路である。リリース識別子を更新し、ソース checkout とは独立して公開成果物を検証
する。

対象リポジトリ以外の object repository は範囲外である。本 Work Item は Outcome、
host 表示、性能、ライフサイクル、ガバナンスの新しい動作を追加しない。

## 受入れ

- workspace package、lock metadata、現在の release/reference 文書、生成 archive が
  v0.2.99 を一貫して識別する。
- 注釈付き v0.2.99 tag は不変でレビュー済み source commit に結び付き、既存 tag は
  変更しない。
- provider Release は安定状態で、manifest、checksum、SBOM、archive、attestation、
  source identity、workflow handoff に結び付く。
- ダウンロードした v0.2.99 成果物が checksum、隔離 fresh-install、v0.2.98 upgrade
  acceptance を通過し、一時 root が正確に清掃される。object repository は変更しない。
- 英語・簡体字中国語・日本語の projection が同期し、最終 Outcome は会話言語に従う。
  性能利益と host 表示確認は未知のまま正しく示す。

## 検証計画

高価な release job の前に format、version、documentation、parity、governance、workspace
check を確認する。その後、レビュー済み main と dispatch 専用 release workflow を使い、
失敗した候補の証拠を保持し、publication handoff を消費してダウンロード成果物の
adopter acceptance を実行する。正確な provider と branch/worktree cleanup evidence を
結び付けてから close する。
