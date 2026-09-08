---
author: AI Cockpit 维护者
title: “WI-698——P1 显式观察上下文边界”
description: “让一次治理判断绑定一组经过验证、按阶段划分的仓库观察上下文。”
audience: [contributor, maintainer, reviewer]
workItemId: WI-698-p1-observation-context-boundary
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-698-p1-observation-context-boundary
---

[English](WI-698-p1-observation-context-boundary.md) · [日本語](WI-698-p1-observation-context-boundary.ja.md)

# WI-698——P1 显式观察上下文边界

## 意图

在不跨越修改扩大快照有效期的前提下完成剩余的 P1-B 架构边界。一次治理判断应使用一组显式、按请求划分的观察阶段事实，而不是让底层辅助函数隐式重新解析仓库身份和快照事实。

## 边界

`RepositoryExecutionContext` 为命名阶段（`before_governance`、`after_execution`、`before_persistence`）生成 `ObservationContext`。上下文绑定仓库身份、可选 Runtime 身份、Contract 身份、源快照摘要、治理配置/策略身份、已解析的仓库观察、项目治理事实、未知项和一致性状态。

`validate_current` 会执行新的边界校验。源文件、绑定身份、Contract 或治理配置发生变化时，旧上下文会被拒绝；上下文也不能复用于另一个生命周期阶段。preflight 治理路径消费此上下文并复用已解析的项目治理事实；兼容包装和既有协议字节保持可用。

## 兼容性与限制

本 WI 不新增 crate、trait、全局缓存、协议字段、治理规则、Outcome 文案、生命周期转换或物理执行策略。上下文不是事务，不能把多文件读取变成原子操作。执行后或持久化前必须重新捕获对应阶段的上下文。不宣称已获得基准性能收益。

## 验证

定向测试覆盖请求内复用、源文件变化、治理配置变化、仓库身份变化、Contract 变化和阶段分离。finish 前仍需通过生命周期 preflight/order 测试和完整 workspace 门禁；终态由 hosted checks 以及 Runtime 的 archive、finalization、close 回执证明。

## 剩余风险

其他生命周期入口仍保留兼容的 root-plus-snapshot 形式，后续迁移必须由独立且有界的 WI 完成。本 WI 不宣称全仓库原子快照或跨请求缓存。
