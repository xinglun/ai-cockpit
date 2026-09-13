---
author: AI Cockpit maintainers
title: "WI-831 — リリース識別子の境界"
description: "不変な成果物識別子と workflow dispatch 識別子を分離します。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-831-release-identity-boundary
lastVerifiedBy: WI-831-release-identity-boundary
---

[English](WI-831-release-identity-boundary.md) · [简体中文](WI-831-release-identity-boundary.zh-CN.md)

# WI-831：リリース識別子の境界

## 意図

WI-830 の dispatch 構文修正をマージした後も、公開処理は不変な成果物のソース
commit と現在の workflow dispatch commit が同一であることを要求していました。
この Work Item では fail-closed の境界を保ったまま、レビュー済みの実行系 commit
から既に作成された不変 tag を公開できるようにします。

## 範囲と判断

dispatch の事前検査では、ローカルの annotated tag がリモート tag と同じ commit
へ peel されることを検証します。公開時は manifest をその tag commit に束ねます。
dispatch workflow revision は別の実行識別子として記録します。どちらの識別子も
相手を書き換えず、v0.2.92 の tag も作り直しません。

## 検証

- ポリシーテストは dispatch のみの公開と、close 式の括弧の整合性を保持します。
- 公開前の境界は、コンパイル前に mutable または移動した tag を拒否します。
- 既存の不変な v0.2.92 を dispatch する前に、現在の main から hosted workflow をレビューします。

## 範囲外

過去の Work Item と receipt、workspace 全体の再実行、無関係な branch や worktree
の清掃、そしてこの Contract の検証前の tag の書き換えや公開は対象外です。
