---
author: AI Cockpit maintainers
workItemId: WI-883-release-v0-2-94-current-main
title: current main からの管理された v0.2.94 リリース
description: Outcome、HCI、四方向収束、Issue #851、インターフェース発見、Rust ツールチェーン作業を同期済み main で検証した後にのみ v0.2.94 を公開する。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
predecessorWorkItemId: WI-871-release-v0-2-94-current-main
lastVerifiedBy: WI-883-release-v0-2-94-current-main
---

# WI-883 — current main からの管理された v0.2.94 リリース

この Work Item は、古い基線の WI-870 と WI-871 のリリース試行を不変履歴と
して保持した後の current main リリース経路です。公開は最後の手順です。完全な
アーカイブ済み Outcome 配信、HCI 対話イベント、四方向収束の証拠、Issue #851
対応、インターフェース発見試行、Rust 1.98.1 と lock 依存関係の更新、候補検証、
レビュー済みマージ、不変なダウンロード成果物の受入れが完了してから、provider
tag または公開 Release を変更します。

公開時も v0.2.93 を N-1 基線として保持します。性能改善やホスト表示確認が未知
の場合は未知のまま記録し、CI の成功やレポート生成だけから完了とは推測しません。
