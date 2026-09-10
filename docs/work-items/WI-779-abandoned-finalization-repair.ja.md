---
author: AI Cockpit maintainers
title: "WI-779 — abandoned finalization 修正"
description: "明示的に close された未 merge の失敗 delivery に正直な終端状態を追加する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
workItemId: WI-779-abandoned-finalization-repair
lastVerifiedBy: WI-779-abandoned-finalization-repair
---

[English](WI-779-abandoned-finalization-repair.md) · [简体中文](WI-779-abandoned-finalization-repair.zh-CN.md)

# WI-779 — abandoned finalization 修正

## Intent と boundary

WI-779 は、provider の Pull Request が merge されずに明示的に close された失敗 delivery
の Runtime resource-finalization 境界を修正する。新しい `abandoned` terminal state は正直な
failure record であり、正確な `unmerged_pull_request` failure code、merge commit がないこと、
branch/worktree が除去済みであることを要求する。これは merged success として projection されない。

PR #760 は close 済みの歴史的な失敗分析入力として保持する。本 Work Item は現在の `main` から
開始し、同 PR や branch を復活させず、WI-774、WI-775、WI-776、WI-778 の immutable record を
書き換えない。

## Scope

- typed Runtime protocol と repository の close/finalization check を拡張する。
- protocol、repository、CLI、closed-documentation promotion の focused regression を追加し、
  invalid な abandoned receipt を fail-closed にする。
- この境界を English、Simplified Chinese、日本語で記録し、reference-parity table に本 Work Item を登録する。

Performance implementation/measurement、release publication、無関係な product behavior、provider API behavior、
user-benefit claim は scope 外である。

## Verification

宣言した workspace verification は `cargo test --locked --workspace` である。focused check は protocol、
repository、CLI、closed-documentation promotion suite を対象とする。hosted quality route では documentation
acceptance、governance integrity、status consistency、および既存の repository gate 全てを通過させる。

