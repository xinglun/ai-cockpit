---
author: AI Cockpit maintainers
workItemId: WI-131-evidence-timestamp
title: 検証証拠 timestamp の fail-closed 検査
description: Outcome または lifecycle 完了前に、検証および retention metadata の不正な RFC3339 timestamp を拒否する。
audience:
  - adopter
  - contributor
  - maintainer
status: implementation
authority: canonical
lastVerifiedBy: WI-131-evidence-timestamp
---

# WI-131 — 検証証拠 timestamp の fail-closed 検査

## Intent

検証証拠は監査可能な記録です。形式だけ存在する不正な timestamp を現在の証拠と
して扱ったり、lifecycle を green に進めたりしてはいけません。

## Boundaries

- v2 envelope の `createdAt` と retention の `createdAt`/`expiresAt` を RFC3339 として検査する。
- Outcome、finish、archive、close で既存の証拠検証を共有する。
- 過去の bytes は保持し、legacy evidence は fresh verification まで historical yellow とする。
- Contract 原文を翻訳せず、retention policy の意味を変更しない。

## Acceptance

- 他の identity/digest 検査が通る有効な v2 timestamp は green のままになる。
- 欠落・不正形式・意味不正の timestamp は green を生成せず、finish/archive/close を停止する。
- repository と CLI の回帰テストで改ざん、archived close、legacy evidence を検証する。
- 英語・簡体字中国語・日本語の Outcome 文書が timestamp の境界を説明する。

## Verification

Focused repository/CLI tests、workspace checks、documentation acceptance の結果は
active Contract と Runtime evidence を参照してください。
