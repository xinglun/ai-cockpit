---
author: AI Cockpit maintainers
workItemId: WI-130-status-closed-projection
title: Closed Work Item status projection
description: archive の事実を変更せず、有効な repository-bound close decision を終端 status へ投影する。
audience:
  - adopter
  - contributor
  - maintainer
status: implementation
authority: canonical
lastVerifiedBy: WI-130-status-closed-projection
---

# WI-130 — Closed Work Item status projection

Runtime は structured close decision を保存しますが、read-only status projection は
以前 archived Summary だけを読み、close 後も `finish_ready` と表示することがありました。
この Work Item は `archived` と `closed` を分離し、検証済み decision だけを projection に使います。

## 境界

- archived Contract、Summary、Outcome、manifest の bytes は変更しません。
- Work Item identity、closed state、confirmed decision state、strict な structured human
  decision が全て検証できた場合だけ `closed` を表示します。
- decision の欠落・不正は unknown として残し、ファイルの存在だけで close を推測しません。

## 受入れ

- archive 後は `archived`、有効な close decision 後だけ `closed` を CLI と repository が返します。
- valid、missing、malformed、foreign、invalid close record の回帰を追加します。
- English、簡体字中国語、日本語の Outcome 文書で終端 projection の境界を説明します。

## 検証

Focused test、workspace check、documentation acceptance の結果は active Contract と Runtime
evidence に記録します。
