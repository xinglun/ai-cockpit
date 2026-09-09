---
author: AI Cockpit maintainers
title: "WI-755——P1 观察上下文后继项"
description: "从当前默认分支完成显式观察上下文边界，不复活旧 PR。"
audience: [contributor, maintainer, reviewer]
workItemId: WI-755-p1-observation-context-successor
status: recovered
authority: human:repository-owner
lastVerifiedBy: WI-758-wi755-finalization-recovery
recoveryDecision: .ai/decisions/WI-755-p1-observation-context-successor.recovery.ccd6ce5cf8f1c2a563578437cc313079462d08a88b04fb8b61b734f54bf237a2.json
---

[English](WI-755-p1-observation-context-successor.md) · [日本語](WI-755-p1-observation-context-successor.ja.md)

# WI-755——P1 观察上下文后继项

## 意图

从当前 `origin/main` 完成剩余的 P1-B 架构边界，不复活失败的 PR #697 或 PR #698。
一次治理判断必须消费一个明确的、请求范围且阶段范围的观察上下文，底层 helper
不得隐式重新观察仓库。

## 边界

`RepositoryExecutionContext` 为 `before_governance`、`after_execution` 和
`before_persistence` 生成 `ObservationContext`。上下文绑定仓库身份、可选 Runtime
身份、Contract 身份及模型身份、源快照摘要、治理配置与策略身份、已解析的仓库观察、
项目治理事实、未知项和一致性。`validate_current` 建立新的边界检查；源文件、绑定身份、
Contract 或治理配置变化都会拒绝旧上下文。一个阶段不能复用于另一个生命周期阶段。

preflight 治理路径消费该上下文并复用已解析的项目治理事实。兼容包装和既有协议字节保持可用。

## 兼容性与限制

本 WI 不新增 crate、trait、全局缓存、协议字段、治理规则、Outcome 文案、生命周期转换或物理执行策略。
上下文不是事务，不能把多文件读取宣称为原子快照。执行后或持久化前必须捕获新的阶段上下文。
本 WI 不宣称已测得性能收益。

## 验证

聚焦测试覆盖请求内复用、源文件变化、治理配置变化、仓库身份变化、Contract 变化、阶段隔离及
Contract/上下文不匹配。还需通过 repository preflight、项目治理、观察器、workspace format、clippy 和完整 workspace 测试。

## 剩余风险

其他生命周期入口仍保留兼容包装，可能继续使用旧的 root 加 snapshot 形式，直到后续独立迁移。
本 WI 不宣称全仓库原子快照或跨请求缓存。

## 合并后恢复边界

PR #735 已通过 hosted checks，并以 `8dcac7ec` 合并。已审阅 PR head 是
`06f7d03f`，而不可变的合并前 finalization receipt 记录的是中间 head
`251c3867`。Runtime 正确拒绝把该范围视为未绑定的 finalization 转换。上面的 recovery
decision 保留这一历史；WI-758 负责新的合并后治理边界。
