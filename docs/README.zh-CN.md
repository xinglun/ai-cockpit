---
author: AI Cockpit maintainers
title: "AI Cockpit 文档"
description: "面向读者的 AI Cockpit 文档入口，用于理解、采用和使用项目。"
audience:
  - adopter
  - contributor
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - documentation_architecture
---

# AI Cockpit 文档

[English](README.md) | [日本語](README.ja.md)

这是 AI Cockpit 面向读者的入口。先按你要完成的事情进入，再阅读定义机器接口的
技术页面；不需要一开始就理解实现细节。

## 从这里开始

- [设计思想](philosophy.zh-CN.md)——为什么必须明确证据和人类决定。
- [架构](architecture.zh-CN.md)——runtime 流程、所有权和边界。
- [功能与边界](capabilities.zh-CN.md)——命令、生命周期、MCP 和恢复。
- [发布与分发](release/distribution.zh-CN.md)——安装方式和发布事实。
- [30 秒命令导览](capabilities.zh-CN.md#功能一览)——当前功能索引。

## 按读者目标选择

| 目标 | 从这里开始 | 读完后应能做到 |
| --- | --- | --- |
| 理解项目 | [设计思想](philosophy.zh-CN.md) → [架构](architecture.zh-CN.md) | 解释证据流和产品边界。 |
| 判断是否采用 | [功能与边界](capabilities.zh-CN.md) → [安装](release/distribution.zh-CN.md) | 选择安装方式，并知道安装不会改变什么。 |
| 开始受治理的工作 | [功能与边界](capabilities.zh-CN.md#运行受治理的-work-item) → [Work Item 规则](work-items/README.zh-CN.md) | 检查、attach、preflight、verify 并关闭有界 Work Item。 |
| 创建治理骨架 | [功能与边界](capabilities.zh-CN.md#创建-work-item-骨架) → [命令参考](reference/commands.zh-CN.md) | 创建 `not_ready` 脚手架并查看仍需人类输入的字段。 |
| 配置 MCP client | [功能与边界](capabilities.zh-CN.md#使用-mcp) → [MCP 分发](release/distribution.zh-CN.md#mcp-与-repository-attach) | 用显式 repository 绑定启动服务并读取结果。 |
| 审查或恢复结果 | [功能与边界](capabilities.zh-CN.md#停止与恢复) → [对抗性验证](security/adversarial-validation.zh-CN.md) | 阅读决定、保留证据并修复停止原因。 |
| 维护或审计系统 | [架构](architecture.zh-CN.md) → [Protocol v1](protocol/v1/specification.zh-CN.md) | 找到所有权、边界和面向机器的 contract。 |

## 技术参考

- [产品边界](architecture/product-boundary.zh-CN.md)
- [Runtime 拓扑](architecture/runtime-topology.zh-CN.md)
- [发布分发架构](architecture/release-distribution.zh-CN.md)
- [版本策略](architecture/versioning.zh-CN.md)
- [Repository Protocol v1](protocol/v1/specification.zh-CN.md)
- [Protocol 兼容规则](protocol/v1/compatibility.zh-CN.md)
- [性能验收](../tests/performance/README.zh-CN.md)
- [实测性能基线](performance/baseline.zh-CN.md)
- [对抗性验证](security/adversarial-validation.zh-CN.md)
- [参考](reference/README.zh-CN.md)——命令、配置和恢复。

## 维护者与审计路线

- [Work Item 规则](work-items/README.zh-CN.md)——本仓库的受治理生命周期。
- [命令参考](reference/commands.zh-CN.md)——精确的 CLI 边界和输出。
- [Protocol v1](protocol/v1/specification.zh-CN.md)——repository storage 与 evidence contract。
- [版本策略](architecture/versioning.zh-CN.md)——Runtime、repository schema 和 migration 边界。
