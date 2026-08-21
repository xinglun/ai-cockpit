---
author: AI Cockpit maintainers
title: "设计思想"
description: "为什么 AI Cockpit 将 repository 事实转换为可供人审查的有界决策。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - design_philosophy
keywords: [ai-cockpit, design-philosophy, evidence, human-control]
---

# 设计思想

## 目的

本页回答：**为什么 AI Cockpit 被设计为治理层，而不是自主 Agent 或工作流引擎？**

## 读者

当你判断产品是否适合自己的开发流程，或想知道为什么某项检查会停止而不是猜测时，
阅读本页。

## 读完之后

你会了解 runtime 背后的原则，也会知道哪些事实由 AI Cockpit 在本地证明，哪些必须
继续由外部系统提供证据。

## North Star

AI Cockpit 服务于经过校准的人机信任。它让预期变更、允许范围、repository 事实、
验证结果和仍需人决定的事项保持可见：

```text
Evidence → Governance Decision → Human Control
```

## 原则

1. **证据优先于自我声明。** 命令、Agent 消息或本地标志本身不是证明。决策来自有类型
   的 repository 事实和记录的证据。
2. **边界必须明确。** Work Item 在实现前声明意图、范围、排除项、权限、验收条件和所需证据。
3. **只观察一次快照。** Git 状态、配置和相关文件被观察一次，并作为 immutable input
   重用。之后的变化是新的事实，不能静默并入旧决策。
4. **失败关闭。** 缺失、过期、矛盾或被篡改的证据会成为 `unknown` 或 `blocked`，不会
   为了方便而成为 pass。
5. **控制与风险相称。** 低风险本地检查保持轻量；受保护 gate 需要更强的身份、证据和
   人类权限。
6. **人类控制必须真实存在。** AI Cockpit 可以解释下一步是否安全，但不能批准未验证的
   变更、认证外部参与者或替代人工审查。
7. **Adapter 保持薄。** CLI 和 MCP 只转换请求与响应；治理规则位于共享 application
   service 和纯 core 中。

## 实际含义

当用户让 Agent“更新文档”时，AI Cockpit 不会把这句话解释为不受限制的工作流。它要求
一个有界的 Work Item，记录 repository 基线，运行声明的检查，并呈现一个人可以继续、
调查、批准、阻止或恢复的决策。

## 应放在哪里

把请求、范围、repository 状态、验证和人类决定放进受治理的 Work Item。把 provider 签名、
SBOM、漏洞扫描或生产审批等领域专属证明交给真正能够产生它的工具或服务。链接这些证据，
但不要重复宣称所有权。

## 停止条件

当请求没有声明边界、证据所有权不明确、受保护操作期间快照发生变化，或有人把本地记录
当作外部控制的证明时，必须停止。应调查缺失的连接，而不是猜测。

## 下一步

1. [架构](architecture.zh-CN.md) — runtime 路径和证据所有权。
2. [功能一览](capabilities.zh-CN.md) — 面向读者的功能概览和详细说明。
3. [产品边界](architecture/product-boundary.zh-CN.md) — 明确哪些责任在外部。

## 技术深度

实现通过 Repository Protocol、有类型的 Work Item 生命周期、immutable repository 快照、
确定性的治理决策、有界 verification plan、内容寻址证据，以及共享 CLI/MCP service 来表达
这些原则。这些机制服务于审查；它们不是通用语义风险检测器、身份提供方、sandbox 或合规证书。
