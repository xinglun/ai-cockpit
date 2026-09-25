---
author: AI Cockpit 维护者
title: WI-1032——跨 WI 独立审查修复
description: 收敛身份、准入、恢复、复用和 MCP 契约的剩余缺口。
workItemId: WI-1032-cross-wi-review-fixes
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: replaced
authority: human-authorized
lastVerifiedBy: WI-1032-cross-wi-review-fixes
---

# WI-1032——跨 WI 独立审查修复

## 意图

Runtime 已将此 Work Item 归档为被 WI-1033 替代，verification claim 为 not_verified。原始 Contract、Summary 和失败的前置条件 evidence 均已保留。下述本地开发 evidence 仅作历史背景，不代表 WI-1032 已通过 Runtime 验证。

## 范围

- 将登记、组合准入、证据、目标合并、资源代次和 Outcome 状态绑定到实际仓库事实。
- 使失效登记可恢复，保留追加式事件历史，并按选定的 provider/outcome 对过滤动作准入。
- 仅在 executable、完整且有界的读取集合、环境和上游 receipt 均被实际观察为未变化时复用组合结果；否则重新执行。
- 为每个 MCP coordination action 明确 provider、Work Item、event 和 consumer 身份语义。
- 保持查询只读、写入显式，并保留普通单 WI 验证的串行路径。
- 对 Sentinel 做候选 Runtime 的只读兼容检查；Sentinel 源码改造留给后续独立 WI。

## 不在范围内

准备或发布版本、tag 变更、安装/升级 Runtime、本 WI 中对 Sentinel 源码或生命周期的写入、全局 Agent/MCP 配置，以及大范围重做协作架构。

## 验收

1. 登记和依赖准入基于已核验的仓库、Contract、branch/head、证据和代次事实；安全暂停时不得启动组合验证。
2. 核验必需参与者/检查和真实目标合并状态；attempt 与清理可恢复；仅按完整观察身份选择可复用节点。
3. 影响报告、恢复和动作准入在中断、代次、provider/outcome 对及传递消费者场景下保持一致。
4. MCP schema 与 handler 对每个 action 定义唯一身份语义，并有 CLI/MCP parity 和 canonical gate 保护。
5. 只读 inspection 不持久化协作状态；影响/发布/协调/恢复通过显式写入口完成。
6. 完成真实多进程与 linked-worktree 验收、普通串行路径、canonical verification、独立 PR 审查、合并和授权清理后才能关闭 WI；停在发布之前。

## 证据与当前状态

Task 1–7 已按串行顺序实现并分别提交。组合、协调存储及 CLI/MCP 查询—写入边界针对性测试通过；gate-manifest 回归、release CLI 构建、文档验收和真实多进程验收也通过。真实验收使用两个 linked worktree；完全重复组合的进程数为 1 → 0；保持 composition JSON 字节不变、只改变实际观察到的环境输入后，进程数回到 1。查询测试在存储不存在和已有记录两种情况下逐项比较协调目录路径与文件字节；只有显式注册和影响报告写入会持久化记录，新的 MCP 进程随后能观察到这些记录。Runtime-bound 完整验证、Sentinel 只读兼容检查、独立 PR 审查、合并和精确清理仍待完成。这些是本地开发证据，不是 Runtime-bound verification 或发布批准。详见[规格](WI-1032-cross-wi-review-fixes/spec.md)、[实施计划](WI-1032-cross-wi-review-fixes/implementation-plan.md)和[独立审查](WI-1032-cross-wi-review-fixes/independent-review.md)。
