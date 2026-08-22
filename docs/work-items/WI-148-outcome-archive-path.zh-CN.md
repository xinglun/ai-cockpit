---
author: AI Cockpit maintainers
title: "WI-148——归档 Outcome 路径投影"
description: "Work Item 归档后保持生成的 Outcome 与面向人交接引用有效。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-148-outcome-archive-path
---

# WI-148——归档 Outcome 路径投影

active Work Item 目录是临时生命周期状态。Work Item 归档时，Runtime 会在写入
archive manifest 和摘要前，将生成的 Outcome、Task Outcome 报告、事件以及
`changedPaths` 引用投影到对应的 archive 路径。这样原始记录和面向人的 handoff
不会指向已经不存在的 active 文件。

该投影只在创建新 archive 时执行。已有历史 archive bytes 保持不可变，不会回填或重写。

证据：`.ai/evidence/WI-148-outcome-archive-path.verification.json`。
关闭决定：`.ai/decisions/WI-148-outcome-archive-path.close.json`。
