---
author: AI Cockpit maintainers
title: "WI-991——WI-990 终态文档投影"
description: "完成 WI-990 关闭后要求的终态文档投影。"
audience: [maintainer, reviewer, contributor]
status: recovered
authority: authorized
workItemId: WI-991-wi990-doc-promotion
lastVerifiedBy: WI-991-wi990-doc-promotion
---

[English](WI-991-wi990-doc-promotion.md) · [日本語](WI-991-wi990-doc-promotion.ja.md)

# WI-991——WI-990 终态文档投影

WI-991 保留为 WI-990 关闭后检查发现六份终态投影仍需生成的不可变失败前置项。
其验证尝试的 Contract 永久保留了错误的命令拼写；recovery
`.ai/decisions/WI-991-wi990-doc-promotion.recovery.json` 和 retirement
`.ai/decisions/WI-991-wi990-doc-promotion.retirement.json` 明确绑定 WI-992 为修正后的
successor。WI-991 不声称验证或关闭。

## 边界

范围包括三份 WI-990 语言页面、三份 reference-parity 台账以及本 Work Item 自身的
三语投影。不修改源码、测试行为、发布逻辑或既有生命周期记录。

## 验收

- 六份 WI-990 终态投影与其 archive、verification 和 close 证据一致。
- 三份 WI-991 页面和 parity 行在关闭前保留本 Work Item 的有边界自投影。
- WI-990 单项投影、全仓 `--check-all`、parity 和状态一致性检查通过。
