---
author: AI Cockpit 维护者
title: WI-1033——跨 WI 审查修复恢复
description: 为选定的跨 WI 审查修复链完成 Runtime 绑定验收与生命周期收尾。
workItemId: WI-1033-cross-wi-review-fixes-recovery
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-1033-cross-wi-review-fixes-recovery
---

# WI-1033——跨 WI 审查修复恢复

## 意图

沿用同一有限的跨 Work Item 审查修复目标，完成 Runtime 验证、独立 PR 审查、合并和精确清理；停在发布之前。

## 恢复链与当前状态

WI-1033 是 WI-1032 的 Runtime 绑定恢复后继项。Runtime 已将 WI-1032 归档为 replaced，verification claim 为 not_verified，并保留其源码记录与失败准入 evidence。实现提交已经存在，但 WI-1033 仍需自己的 Contract 绑定验证；既有本地测试报告不是 Runtime verification receipt。

较早的 WI-1031 归档项有有效的历史验证，但仍待人工 close。其字节保持不可变；只有 Runtime 收齐终态 evidence 后，才能通过所选 successor lineage 处理。

## 范围

- 重新核验受信登记/组合身份、必需检查完整性、崩溃与活动锁安全、证据边界、按 provider 区分的依赖、CLI 实际复用、MCP 身份 parity 及只读查询/写入边界。
- 证明真实多进程 linked-worktree 行为，并保留普通单 WI 串行执行。
- 使用候选 CLI 检查 Sentinel，且不写入其源码、Contract/evidence、生命周期 Runtime 或协调存储。
- 维护英文、简体中文和日文投影与 reference parity。

## 不在范围内

Task 8（跨提交的 Runtime snapshot binding）按要求作为后续独立串行 WI。Runtime 升级、发布准备、tag 变更、公共发布、Sentinel 写入及改写历史记录均不在范围内。

## 验收与验证

参见[规格](WI-1033-cross-wi-review-fixes-recovery/spec.md)和[实施计划](WI-1033-cross-wi-review-fixes-recovery/implementation-plan.md)。八个必需场景均声明了明确的预期结果和验证计划。文档 canonical gate 为 `bash tests/docs/documentation_acceptance.sh`。

## 交付边界

当前状态为进行中，尚未完成验证，也不代表可以发布。完成前仍需新的 Runtime 绑定验证、独立审查、hosted CI、合并、精确清理，以及 Runtime 实际允许的历史 lineage close。是否发布由人决定。
