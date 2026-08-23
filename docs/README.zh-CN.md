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
- [30 秒开始](getting-started/30-second-start.zh-CN.md)——安全 inspect 并 attach 一个 repository。

## 规范读者路线

普通读者路线采用与参考源相同的 goal-first 结构，但只链接本 Runtime 的当前文档：

- [当前路线](current/README.zh-CN.md)——默认路线地图。
- [快速开始](getting-started/README.zh-CN.md)——安装、验证、attach 和连接 Agent。
- [功能](features/README.zh-CN.md)——功能概览与责任边界。
- [运维](operations/README.zh-CN.md)——生命周期、恢复、升级和 Release 验收。
- [参考](reference/README.zh-CN.md)——精确命令、配置和输出。

## 按读者目标选择

| 目标 | 从这里开始 | 读完后应能做到 |
| --- | --- | --- |
| 理解项目 | [设计思想](philosophy.zh-CN.md) → [架构](architecture.zh-CN.md) | 解释证据流和产品边界。 |
| 判断是否采用 | [功能与边界](capabilities.zh-CN.md) → [安装](release/distribution.zh-CN.md) | 选择安装方式，并知道安装不会改变什么。 |
| 开始受治理的工作 | [首个 Work Item](getting-started/first-work-item.zh-CN.md) → [Work Item 规则](work-items/README.zh-CN.md) | 绑定审查资源、preflight、verify、展示 Outcome、完成资源收尾并 close 有界 Work Item。 |
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
- [面向人的 Outcome](reference/outcome-report.zh-CN.md)——可读结果、证据、风险和下一步。
- [最终替代验收](reference/final-replacement-acceptance.zh-CN.md)——conformance 和无复制发布边界。
- [企业治理](security/enterprise-governance.zh-CN.md)——策略层、委托证据、保留和审计导出。
- [安全与威胁模型](security/threat-model.zh-CN.md)——部署边界与安全假设。
- [漏洞报告](security/vulnerability-reporting.zh-CN.md)——支持版本和私密报告路径。

## 维护者与审计路线

- [Work Item 规则](work-items/README.zh-CN.md)——本仓库的受治理生命周期。
- [命令参考](reference/commands.zh-CN.md)——精确的 CLI 边界和输出。
- [Protocol v1](protocol/v1/specification.zh-CN.md)——repository storage 与 evidence contract。
- [版本策略](architecture/versioning.zh-CN.md)——Runtime、repository schema 和 migration 边界。
