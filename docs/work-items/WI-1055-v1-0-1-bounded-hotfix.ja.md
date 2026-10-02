---
author: AI Cockpit maintainers
workItemId: WI-1055-v1-0-1-bounded-hotfix
title: 限定修正版の例外記録
description: WI-1052 と issue 1004 の限定修正版、Runtime の開始拒否、実テスト、未完了の公開証拠を記録します。
audience: [maintainer, reviewer]
status: in-progress
authority: user-authorized-project-bootstrap-exception
lastVerifiedBy: pending-exact-head-acceptance
---

[English](WI-1055-v1-0-1-bounded-hotfix.md) · [简体中文](WI-1055-v1-0-1-bounded-hotfix.zh-CN.md)

# WI-1055 限定修正版の例外記録

状態: ユーザーが認めたプロジェクト限定の bootstrap 例外に基づく実装中。Runtime が開始を許可した、検証済み、クローズ済み、マージ済み、公開済み、という意味ではありません。

基点は main `78ae7240aac016f005fcf4ced61fcb251ea20eb1` です。旧候補 `befdbd11d60af5c01d40ec62497aa9c8b46ba0ce` は参照のみとし、旧 WI-1053 の active Contract、Summary、amendment receipt、decision を移しません。

対象は WI-1052 の保留中 amendment review request の生成（自動承認はしない）と、Issue #1004 の checkout 間 closeout 復元、履歴 Contract の raw bytes 結合、atomic rollback です。Task9、旧 WI の追認、.ai/ 全体の除外、Sentinel 側のリソース変更は対象外です。

Runtime `1.0.1-rc.1` は WI-1055 の scaffold を作成しましたが、開始を `archived_work_item_scope_conflict:WI-1042-contract-amendment-environment-drift, archived_work_item_scope_conflict:WI-1043-amendment-review-admission-fix` として拒否しました。scaffold は `not_ready` のままです。この文書は説明であり、Runtime receipt や本人確認済み人間レビューではありません。

新しいブランチで WI-1052 CLI 4/4、#1004 repository 3/3、CLI 5/5、MCP 1/1、repository ライブラリ 54/54 の定向テストが通過しました。Sentinel formatter の scope を正確なパスへ修正した後の実 preflight は、履歴の六つの復元ファイルが残っていても、`scope_exceeded` ではなく blocker なしの yellow／人間レビュー待ちでした。したがって以前の red 状態だけから製品欠陥を断定しません。

完全な workspace 検証、PR CI、独立レビュー、Mac arm64 のダウンロード・checksum・インストール・doctor、公開は未完了です。`v1.0.1-rc.1` の tag と資産は上書きしません。Mac のみの証拠しかない場合は新しい prerelease とし、安定版や他プラットフォームの検証済みを主張しません。
