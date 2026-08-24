---
author: AI Cockpit maintainers
title: "WI-257——close 后 promotion 恢复"
workItemId: WI-257-post-close-promotion-recovery
description: "从干净的当前基线恢复 typed close 后文档 promotion，且不重写失败 predecessor。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-257-post-close-promotion-recovery
authority: canonical
---

# WI-257——close 后 promotion 恢复

WI-257 从当前 default-branch 基线重新交付 repository-owned close 后文档
orchestrator。WI-256 与已关闭的 PR #208 保留为仓库外的不可变失败交付历史；本 WI
既不导入其 `.ai` records，也不把它们表述为仓库终态 truth。

## 验收边界

- typed plan 绑定 repository identity、准确同步的 `origin/main`、approved close、
  sequence-2 finalization、archive/evidence identity，以及六个受控文档路径的准确
  before/after digest。
- stale 或 descendant revision、foreign 或 malformed identity、重复或未知 JSON
  字段、symlink/nonregular 输入输出、dirty 或 partial projection、unexpected path
  都会在写入前 fail closed；对已是 current 的 plan 重复 apply 是 deterministic no-op。
- 隔离的 bare-origin regression 通过 `HEAD` 宣告 `main`，使 clone 覆盖 orchestrator
  使用的同一 default-branch identity。
- WI-255 的三份 Work Item 与三份 reference-parity projection 变为 `Implemented`，
  且不修改任何不可变 `.ai` lifecycle byte。

## 生命周期交接

仓库流程为：

```text
close → visible Outcome → post-close plan/apply → check-all → terminal CI
```

WI-257 在自身 verified close 前保持条件状态。parity ledger 列出未来 archived
Contract、verification evidence、finalization chain 与 close receipt 路径；Runtime
创建它们之前，文档绝不声称这些记录已经存在。

## 参考

- [Agent workflow](../reference/agent-workflow.zh-CN.md)
- [Commands](../reference/commands.zh-CN.md)
- [Reference parity](../reference/reference-parity.zh-CN.md)
- [失败 predecessor PR #208](https://github.com/xinglun/ai-cockpit/pull/208)
