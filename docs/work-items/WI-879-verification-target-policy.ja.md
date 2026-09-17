---
author: AI Cockpit maintainers
workItemId: WI-879-verification-target-policy
title: 検証ターゲットキャッシュ方針の置換修正
description: 共有の非 incremental Cargo 検証ターゲット方針を新しいレビュー対象ブランチへ引き継ぎ、WI-878 アーカイブ後に見つかった quality gate 回帰を修正する。
audience: [maintainer, reviewer]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-879-verification-target-policy
---

[English](WI-879-verification-target-policy.md) · [简体中文](WI-879-verification-target-policy.zh-CN.md)

# WI-879 — 検証ターゲットキャッシュ方針の置換修正

この successor は最新のリモート main から WI-878 の実装を新しいレビュー
ブランチへ引き継ぎ、WI-878 のアーカイブ後に quality gate が検出した回帰を
修正します。Cargo 検証は `CARGO_INCREMENTAL=0` と安定したユーザーキャッシュ
のターゲットディレクトリを使い、依存関係を再利用しながら検証ごとの増分ツリー
増加を防ぎます。Cargo 以外のコマンド、対象リポジトリ、Issue #851 の復旧、
リリース公開は対象外です。

WI-878 のアーカイブ証拠は不変のまま保持し、この Work Item では置換実装、
ドキュメント投影、新しい quality evidence のみを記録します。
