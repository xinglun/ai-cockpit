---
author: AI Cockpit maintainers
workItemId: WI-1068-v1-0-1-stable-release
title: v1.0.1 安定版リリース
description: 既存のレビュー済み系列から v1.0.1 安定版を提供し、公開 artifact とインストールを受け入れ確認する。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1068-v1-0-1-stable-release
---

[English](WI-1068-v1-0-1-stable-release.md) · [简体中文](WI-1068-v1-0-1-stable-release.zh-CN.md)

# WI-1068 — v1.0.1 安定版リリース

この Work Item は既存の v1.0.1 系列を正式な安定版として提供し、公式 artifact を検証してローカル CLI/MCP を更新する。

## 境界

- 製品ソースの変更は宣言された16パスに限定する。三言語ページと parity 行は必須の Work Item ガバナンス投影である。
- 実在する v1.0.0 安定版を N-1 の前版とし、完全な N-1 acceptance は x86_64 Linux のみで実行する。
- v1.0.1-rc.2 は独立した immutable prerelease として保持する。新しい安定版 tag は merge と Runtime の新鮮な公開許可後にのみ作成する。既存の tag、Release、asset は上書きしない。
- 既存 release workflow は4対象の candidate smoke と段階的 N-1 検査に合格してから Release を公開する。公開 download/install acceptance と macOS ローカル CLI/MCP 検査は公開後に行う。

ソース候補は PR #1022 でレビュー中である。CI、公開、公開後 acceptance、ローカルインストール、cleanup は各段階の最新 evidence を確認するまで保留し、完了したとは記載しない。
