---
author: AI Cockpit maintainers
title: "WI-993 — WI-867 终态文档投影"
description: "修正 WI-867 自身终态检查仍为过时状态后的三语终态文档投影。"
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: authorized
workItemId: WI-993-wi867-doc-promotion
lastVerifiedBy: WI-993-wi867-doc-promotion
---

[English](WI-993-wi867-doc-promotion.md) · [日本語](WI-993-wi867-doc-promotion.ja.md)

# WI-993 — WI-867 终态文档投影

WI-993 完成 WI-867 的受控终态投影。它保留 WI-867 的 archive、verification 和
close 证据，只修改三语文档投影与 parity 台账。

## 边界

范围仅包括 WI-867 的三语页面、本 Work Item 的三语页面、三份 reference-parity
台账，以及 Runtime 生成的 `.ai/` 证据。源码行为、发布行为和历史治理字节不在范围内。

## 验收

- WI-867 的三语页面和 parity 行与不可变终态证据一致。
- WI-993 在 close 前保留受控的三语自身投影。
- 单项 WI-867 投影、仓库级 `--check-all`、parity 和状态一致性检查全部通过。
