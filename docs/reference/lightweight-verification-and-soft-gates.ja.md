---
author: AI Cockpit maintainers
title: "軽量検証とソフトゲート"
description: "必須のガバナンス制御を弱めず、証拠に基づき検証強度を比例配分する。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-348-reference-verification-operation-policy
---

# 軽量検証とソフトゲート

[English](lightweight-verification-and-soft-gates.md) · [简体中文](lightweight-verification-and-soft-gates.zh-CN.md)

AI Cockpit はリポジトリの事実、Work Item Contract、段階、適用ポリシーから
検証ルートを選びます。`light`、`standard`、`strict` は検証強度であり、
assurance のレベルでも権限でもありません。

## ルール

- ルートはチェックを追加できます。キャッシュを再利用できるのは、内容、
  diff、環境、Runtime、ポリシー、リポジトリ、Work Item、段階の全バインドが
  一致する場合だけです。
- 依存関係の計画は決定的です。循環、壊れたノード、未知の依存関係を完了と
  みなさず、`partial` または `unknown` のまま影響範囲をエスカレーションします。
- エスカレーションは単調です。`light → standard → strict` は必要な作業を
  増やせますが、コスト、再利用、provider のヒントで要件を下げません。
- soft、skip、advisory の観測は証拠に残します。欠落、stale、矛盾、保護対象の
  証拠を緑に変えることはありません。

標準ゲートは明示的なリポジトリコンテキストで評価します。

```sh
ai-cockpit gate --repo /path/to/repository --contract .ai/work-items/active/WI.contract.json
```

ゲートの receipt はルーティングの説明であり、実行トークンではありません。
Hosted CI、release、provider、enterprise assurance は別の委譲境界です。
[Governance profiles](governance-profiles.ja.md) と[Verification semantics](verification-semantics.ja.md)も参照してください。

## オブジェクトプロジェクトへの継承

すべての adopter は同じ共有 Runtime と明示的な `--repo` バインドを使います。
ルートと証拠は各リポジトリに分離され、グローバルな current project や Work
Item はありません。軽量ルートは比例した検証強度であり、Contract、人手レビュー、
スコープ、証拠完全性を省略する許可ではありません。
