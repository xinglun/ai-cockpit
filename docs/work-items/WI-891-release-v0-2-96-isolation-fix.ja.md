---
author: AI Cockpit maintainers
workItemId: WI-891-release-v0-2-96-isolation-fix
title: v0.2.96 リリース検証分離の修正
description: 不変の v0.2.95 候補で判明した明示的な検証ターゲットディレクトリの伝播を修正する。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-891-release-v0-2-96-isolation-fix
---

# WI-891 — v0.2.96 リリース検証分離の修正

この successor は不変の v0.2.95 失敗候補の履歴を保持し、v0.2.96 公開前に
Runtime の環境境界を修正する。固定された実行ファイルで検証するとき、
adopter 受入れルートが明示した `CARGO_TARGET_DIR` を維持しなければならない。

## 受入れ境界

- v0.2.95 を不変のまま保持し、対象リポジトリを変更しない。
- 1 件の PR と hosted checks で Runtime 修正をレビュー・マージする。
- HOME、XDG_CONFIG_HOME、CARGO_HOME、target ルートの分離とクリーンアップを
  ダウンロード成果物の新規インストールおよび N-1 アップグレードで確認してから v0.2.96 を公開する。
- ホスト表示と性能の未知項目を維持し、この WI はリリース分離の阻塞だけを修正する。

## 検証計画

`CARGO_INCREMENTAL=0` と共有検証ターゲットで定向環境テストを実行し、宣言された
リリース・ドキュメントゲートを続けて実行する。v0.2.95 の失敗ワークフローを外部証拠として保持し、メッセージ配信のために検証を再実行しない。

