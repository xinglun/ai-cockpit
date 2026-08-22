---
author: AI Cockpit maintainers
workItemId: WI-130-status-closed-projection
title: Closed Work Item 状态投影
description: 在不重写归档事实的前提下，把有效 repository-bound close decision 投影为终态。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-133-docs-truth
---

# WI-130——Closed Work Item 状态投影

Runtime 已经写入结构化 close decision，但只读 status 过去只读取归档 Summary，
成功 close 后仍可能显示 `finish_ready`。本 Work Item 区分 `archived` 与 `closed`，
并且只把经过验证的 close decision 作为投影依据。

## 边界

- 保留归档 Contract、Summary、Outcome 和 manifest bytes。
- 只有 Work Item identity、closed state、confirmed decision state 和严格结构化人工
  决定都通过验证后，才投影 `closed`。
- 缺失或无效 decision 只能显示未知，不能从文件存在推断已关闭。

## 验收

- archive 后 `work-item status` 和 repository projection 显示 `archived`，有效 close
  decision 后才显示 `closed`。
- CLI 与 repository 回归测试覆盖有效、缺失、格式错误、foreign 和无效 close 记录。
- English、简体中文、日本語 Outcome 文档说明终态投影边界。

## 验证

以 archived Contract、verification evidence、close decision 与 Runtime evidence 中的
focused tests、workspace checks 和文档验收为准。证据为
`.ai/evidence/WI-130-status-closed-projection.verification.json` 与
`.ai/decisions/WI-130-status-closed-projection.close.json`。
