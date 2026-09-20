---
author: AI Cockpit maintainers
title: "WI-953 — リリース後の文書投影昇格"
description: "v0.2.102 のリリース終結後に、close 済み WI-952 の文書投影を昇格する。"
audience: [maintainer, reviewer, contributor]
status: recovered
authority: human:xinglun
workItemId: WI-953-post-release-doc-promotion
lastVerifiedBy: WI-953-post-release-doc-promotion
---

[English](WI-953-post-release-doc-promotion.md) · [简体中文](WI-953-post-release-doc-promotion.zh-CN.md)

# WI-953 — リリース後の文書投影昇格

## 目的

v0.2.102 のリリース経路完了後、close 済み WI-952 の人間向け文書と parity
投影を昇格する。

## 境界

この Work Item は生成済みの終結記録と人間向け文書投影だけを変更する。
Runtime の動作、release asset、object repository は対象外である。

## 受入れ

- WI-952 の三言語ページと parity 行が archive、finalization、close の状態を
  正確に示す。
- 文書検査は project verification subprocess を起動しない。

## 置換記録

この経路は検証前に置換された。不変の Contract が documentation-only の境界と
矛盾する無条件の Cargo workspace verification command を宣言していたためである。
archive bytes は保持され、有効な retirement receipt は
`.ai/decisions/WI-953-post-release-doc-promotion.retirement.json`、束縛された
successor は WI-954 である。WI-953 について verification や close は主張しない。
