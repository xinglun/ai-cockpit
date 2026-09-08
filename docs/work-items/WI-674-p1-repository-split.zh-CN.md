---
author: AI Cockpit maintainers
title: "WI-674——P1 Repository 职责拆分"
description: "在保持公共行为和持久化兼容的前提下拆分 cockpit-repository 内部职责。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-674-p1-repository-split
status: in_progress
authority: authorized
lastVerifiedBy: WI-674-p1-repository-split
---

[English](WI-674-p1-repository-split.md) · [日本語](WI-674-p1-repository-split.ja.md)

# WI-674——P1 Repository 职责拆分

## 意图

在不改变公共 API、wire format、持久化布局、错误行为、授权语义或生命周期判定的
前提下，拆分 `cockpit-repository` 的内部职责。这是结构性重构，不宣称已经测量到
性能或认知收益。

## 边界与依赖图

| 模块 | 职责 | 主要依赖与消费者 |
| --- | --- | --- |
| `lifecycle.rs` | Work Item 入口、checkpoint、preflight、finish、verification、recovery 和生命周期边界辅助函数 | `lib.rs` 的共享 Repository 类型及 governance/evidence 辅助函数；状态/readiness 投影；evidence store；Outcome 投影辅助函数 |
| `evidence_store.rs` | 可复用 receipt 存储、有效性绑定、有界读取、原子发布和 capability filesystem 辅助函数 | Protocol digest 与 Repository identity；由 lifecycle 和 execution-context 使用；持久化路径与字节保持不变 |
| `execution_context.rs` | Repository/runtime 执行上下文、可执行文件身份、staging、shebang 与环境身份，以及 verification reuse 评估 | Git snapshot 和 evidence-store binding；由 verification 和 lifecycle 使用；reuse 规则保持不变 |
| `status_projection.rs` | Repository status、readiness、worktree 拓扑、历史债务和 archive-close 投影 | Git snapshot 与 archive/history reader；由 lifecycle 入口和 status caller 使用；投影保持只读 |
| `lib.rs` | 公共 API、共享 protocol 类型、稳定 re-export 和跨职责辅助函数 | re-export 模块 API，并保留既有序列化和持久化契约 |

依赖方向保持保守：拆出的模块使用 `lib.rs` 中的共享 Repository 定义；`lib.rs` 只
重新导出既有公共表面，并以受限内部导入方式满足原有跨模块调用。没有引入新 crate
或框架。

## 兼容性

基线为远程默认分支 `origin/main` 的
`0b92a420ad76c5f9ce5ea80ae6c6870fcd45b130`。本次变更移动既有实现，不增加协议字段。
公共函数、序列化产物、错误路径、持久化路径和状态语义预期保持字节及行为兼容；最终
仍需以下完整 workspace 与 hosted 检查确认。

## 验证证据

模块移动后以下定向行为测试通过，共 107 个：lifecycle entry/order；recovery
decision/events/revalidation；receipt store；evidence assurance；repository context；
verification context；verification service；status projection。

当前尚未验证：完整 `cargo test --locked --workspace`、格式检查、Clippy、文档/parity
检查、Runtime `verify`、hosted PR 检查及终态 lifecycle 记录。目前没有在纯移动重构中
发现行为缺陷。若发现缺陷，必须单独记录并按治理流程处理，不能隐藏在本结构性 WI 中。

## 排除范围

Outcome P0-A/P0-B 行为、P1-A 认知收益评估、P2 首次使用文档、未来治理原则 WI、其他
agent 的 worktree，以及用户全局 Agent/MCP 配置均不属于本 Work Item。
