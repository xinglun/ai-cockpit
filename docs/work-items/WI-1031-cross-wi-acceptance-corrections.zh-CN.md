---
author: AI Cockpit 维护者
title: WI-1031——跨 WI 验收修复
description: 修复独立验收发现的协作身份、证据、动作准入和 Outcome 缺口。
workItemId: WI-1031-cross-wi-acceptance-corrections
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: close_pending
authority: human-authorized
lastVerifiedBy: WI-1031-cross-wi-acceptance-corrections
---

# WI-1031——跨 WI 验收修复

## 意图

Runtime 当前将此归档项投影为历史验证通过、人工 close 待处理。其选定恢复链经 WI-1032 继续到 WI-1033。在 Runtime 记录准确且获准的 lineage close 前，不要将归档项表示为已终态关闭。

## 范围

- 根据实际仓库、执行器、Contract 和依赖事实计算组合复用身份及必需检查覆盖。
- 仅接受绑定当前提供者身份且成功的有效验证证据，并核实真实目标合并事实。
- 使影响登记可恢复、准入按成果过滤，并覆盖传递依赖。
- 保留历史组合记录，同时分别表达当前过期状态、临时组合、目标合并和安全暂停。
- 保留普通单 WI 串行执行能力。

## 不在范围内

发布、tag 变更、制品发布、Runtime 升级、改写 WI-1030 不可变历史、全局 Agent/MCP 配置及无关协作架构重做。

## 验收

1. 实际工具链或环境变化时，旧调用方 JSON 不能授权复用；无法观察的执行输入会禁用复用。
2. 命令启动前必须覆盖必需场景、参与者、组合顺序和依赖闭包；消费者 worktree 可针对解析后的 `main`。
3. 验证依赖只接受绑定当前提供者身份的受支持成功 receipt；`MergedTarget` 反映真实目标集成。
4. 登记／事件中断可恢复；同 WI 动作按 outcome 过滤，影响沿三层依赖传播。
5. Outcome 保留历史但标示过期，区分临时组合与目标合并，并与暂停准入一致。
6. 普通单 WI 工作流继续串行运行，无需跨 WI 协调状态。

## 证据

目前已通过的针对性证据：组合／复用（17 项测试）、协作准入／投影（25 项测试）、协调存储／恢复（14 项测试）、repository gate manifest 回归，以及使用两个 linked worktree 的真实多进程验收。该验收记录了两个登记、一个去重后的影响事件，以及首次组合启动 1 个验证进程、完全相同输入再次组合启动 0 个进程。其他覆盖包括可执行文件／环境变化负例、选择性和传递性失效、执行前拒绝无效依赖顺序，以及普通单 WI 串行验证。全工作区测试、Clippy、完整 Runtime/Cargo/CI canonical gate、独立 PR 审查、合并和清理仍待完成。本 WI 仍停在发布之前。详见[规格](WI-1031-cross-wi-acceptance-corrections/spec.md)和[实施计划](WI-1031-cross-wi-acceptance-corrections/implementation-plan.md)。
