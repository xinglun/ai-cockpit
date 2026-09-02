---
author: AI Cockpit maintainers
title: "WI-517——WI-516 终态文档晋级"
description: "在不改写不可变治理记录的前提下，晋级已关闭 WI-516 的读者文档投影。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-517-wi516-doc-promotion
lastVerifiedBy: WI-517-wi516-doc-promotion
---

[English](WI-517-wi516-doc-promotion.md) · [日本語](WI-517-wi516-doc-promotion.ja.md)

## 目标

将已关闭 WI-516 的 Work Item 与 parity 投影从条件注册晋级为有终态
证据支持的状态。helper 必须是确定性的，不得改写 WI-516 的 Contract、
Summary、Outcome、Events、verification、finalization 或 close bytes。

## 范围

- `docs/work-items/WI-516-reference-file-comparison-batch-34.md`
- `docs/work-items/WI-516-reference-file-comparison-batch-34.zh-CN.md`
- `docs/work-items/WI-516-reference-file-comparison-batch-34.ja.md`
- `docs/reference/reference-parity.md`
- `docs/reference/reference-parity.zh-CN.md`
- `docs/reference/reference-parity.ja.md`
- WI-517 三语读者记录。

## 验收

- `promote_closed_work_item.py --repo <repo> --check-all` 不再报告 WI-516
  投影过期。
- WI-516 页面包含 `status: implemented` 与准确终态 evidence 路径；三语
  parity 行为 `Implemented` 并链接 archive Contract、verification、
  finalization 和 close 记录。
- 文档、parity 与 Work Item status consistency 检查通过。
- WI-516 Runtime 生成记录保持字节不变。
- 不修改 Runtime 源码、参考源实现、对象/adopter 工程、发布内容或全局
  Agent/MCP 配置。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `AI_COCKPIT_REFERENCE_ROOT=/Users/sei-rinn/dev/workspace_python/ai-cockpit-template bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`

这是 close 后的文档投影工作；生成 receipt 保持不可变。
