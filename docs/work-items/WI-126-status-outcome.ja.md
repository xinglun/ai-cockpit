---
author: AI Cockpit maintainers
workItemId: WI-126-status-outcome
title: Read-only Work Item status と human handoff projection
description: CLI と MCP が一つの evidence-bound status/Outcome projection を使う。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: documentation-acceptance
---

# WI-126 — Read-only status と human handoff

この Work Item は request-scoped な `work-item status` projection を追加し、
CLI/MCP の人間向け Outcome を一つの validated source に揃えます。scheduler、
global current project、または第二の governance decision engine は作りません。

実装範囲は lifecycle、governance、activity health、fact count、blocker、evidence、risk、
permission、unknown、diagnostic、source digest、CLI/MCP の先頭
`Outcome: 🔴/🟡/🟢` handoff、Contract 原文の byte 保持、historical/invalid evidence の
非 green 投影、三言語 WI-125 field mapping/parity baseline です。

最終 twenty-dimension aggregator と外部 assurance は後続境界です。Status/Outcome は
read-only projection であり、認可そのものではありません。
