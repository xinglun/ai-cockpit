---
author: AI Cockpit maintainers
title: "WI-987 — WI-986 documentation promotion"
description: "WI-986 documentation promotion の失敗試行と明示的に結び付いた後継を保持する。"
audience: [maintainer, reviewer, contributor]
status: recovered
authority: authorized
workItemId: WI-987-wi986-doc-promotion
lastVerifiedBy: WI-987-wi986-doc-promotion
---

[English](WI-987-wi986-doc-promotion.md) · [简体中文](WI-987-wi986-doc-promotion.zh-CN.md)

# WI-987 — WI-986 documentation promotion attempt

WI-987 は不変の failed predecessor として保持される。宣言された helper
verification command が終了済み Work Item の identity を誤って指定したため、
Runtime は生成済み Contract を書き換えず successor decision を記録した。
replacement archive は
`.ai/work-items/archive/WI-987-wi986-doc-promotion.archive.json`、recovery
binding は `.ai/decisions/WI-987-wi986-doc-promotion.recovery.json` であり、
修正後の verification と最終 projection は WI-988 が担当する。WI-987 は
verification または close を主張しない。

## Historical boundary

- failed verification evidence は
  `.ai/evidence/WI-987-wi986-doc-promotion.verification-attempt.47cf4f9c393b8b2a2ec989084cd6c7d7b70bed0407b7b354d240118d8c4bdf8d.json` に保持される。
- recovery record は WI-988 を successor として明示的に bind する。
- WI-987 は verification、completion、close を主張しない。
