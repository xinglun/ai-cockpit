---
author: AI Cockpit maintainers
title: "WI-186 — v0.2.23 post-release public adopter acceptance"
workItemId: WI-186-release-v0-2-23-post-release-acceptance
description: "不変な公開 v0.2.23 Runtime が新しい adopter と N-1 upgrade を統治できることを記録します。"
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-186-release-v0-2-23-post-release-acceptance
---

# WI-186 — v0.2.23 post-release public adopter acceptance

WI-186 は、公開 v0.2.23 Runtime をインストールした後の次サイクルの基線を
記録します。ダウンロードした Release binary だけを使用し、Cargo build、
`cargo run`、workspace binary、またはローカル `target/` fallback は使用し
ません。

不変な Runtime identity は
`.ai/evidence/external/v0.2.23/adopter/runtime.json` に記録します。公開
adopter run と v0.2.22 → v0.2.23 N-1 upgrade run は、それぞれの
`acceptance.json`、close receipt、isolation manifest、cleanup receipt、
`SHA256SUMS` を保持します。

Adopter evidence は、隔離された repository で attach、Agent discovery、
evidence reuse、`first-adopter-smoke` の `not_ready` 境界、完全な Work Item
lifecycle が動くことを示します。HOME と XDG configuration は変更されず、
temporary root と Cargo root は分類され、終了時に明示的に cleanup されます。

この Work Item は Release、tag、または historical evidence を書き換えません。
公開された事実を記録し、次の Work Item がインストール済み v0.2.23 Runtime
だけを governance interface として使えるようにします。

Evidence: `.ai/evidence/external/v0.2.23/adopter/acceptance.json` と
`.ai/evidence/external/v0.2.23/upgrade/acceptance.json`。

[English](WI-186-release-v0-2-23-post-release-acceptance.md) ·
[简体中文](WI-186-release-v0-2-23-post-release-acceptance.zh-CN.md)
