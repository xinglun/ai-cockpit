---
author: AI Cockpit maintainers
title: "参考"
description: "面向用户的命令、配置和恢复参考。"
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - reference_index
---

# 参考

先阅读[当前读者路线](../current/README.zh-CN.md)和功能导览，再使用以下页面。路线入口把普通用户旅程
与精确的机器接口细节分开：

- [快速开始](../getting-started/README.zh-CN.md)——安装和首次 attach。
- [功能](../features/README.zh-CN.md)——能力目标和边界。
- [运维](../operations/README.zh-CN.md)——生命周期、恢复、升级和验收。

- [命令参考](commands.zh-CN.md)——命令分组、必需绑定和输出行为。
- [配置参考](configuration.zh-CN.md)——`.ai/cockpit.toml`、profile 和生成记录。
- [排查与恢复](troubleshooting.zh-CN.md)——停止状态和安全下一步。
- [面向人的 Outcome](outcome-report.zh-CN.md)——可读结果、风险、证据和下一步。
- [治理配置级别](governance-profiles.zh-CN.md)——与风险相称的 Light/Standard/Strict 路由及 assurance 边界。
- [如何阅读 Cockpit 状态](how-to-read-cockpit-status.zh-CN.md)——面向人的颜色、证据和下一步阅读顺序。
- [Agent 工作流与评审边界](agent-workflow.zh-CN.md)——Work Item、Outcome、发布与安全规则的本工程适配。
- [Work Item 编写指南](work-item-style-guide.zh-CN.md)——由人拥有的 intent、scope、验收和可执行验证指导。
- [C# 技术栈适配](csharp-adaptation.zh-CN.md)——Rust 原生的 C#/.NET adopter 映射与明确的安装边界。
- [Android fixture 适配](android-fixture-adaptation.zh-CN.md)——逐文件的 Rust 原生 Android fixture 映射与明确的安装边界。
- [Flutter fixture 适配](flutter-fixture-adaptation.zh-CN.md)——逐文件的 Rust 原生 Flutter fixture 映射与明确的安装边界。
- [Verification 路线](verification-route.zh-CN.md)——类型化阶段、正交 tier/assurance、计划、回执和 CI 边界。
- [实现知识](implementation-knowledge.zh-CN.md)——确定性、证据绑定的记录和查询边界。
- [输入信任数据流](input-trust-dataflow.zh-CN.md)——来源分类和 fail-closed 输入处理。
- [已安装 Runtime 生命周期](installed-lifecycle.zh-CN.md)——共享 Runtime 的安装、attach、升级和回滚边界。
- [指令可追溯性](instruction-traceability.zh-CN.md)——source path、Work Item、证据和关闭之间的关系。
- [Verification 证据复用 Runtime](verification-evidence-reuse-runtime.zh-CN.md)——有限规划、受保护节点和身份绑定 receipt。
- [Verification 证据复用决策](verification-evidence-reuse.zh-CN.md)——新鲜度绑定、失效和可测量的调用减少。
- [Verification fixture 边界](verification-fixture-boundary.zh-CN.md)——隔离本地 fixture 及其证据限制。
- [Work Item Intelligence 集成边界](wiii-v2-integration-audit.zh-CN.md)——只读 Rust 投影及非 wire 兼容边界。
- [Work Item Intelligence 性能基线](work-item-intelligence-performance-baseline.zh-CN.md)——不授予治理权限的可复现本地观测。
- [Work Item 生命周期关闭](work-item-lifecycle-closure.zh-CN.md)——评审合并、归档、精确清理与恢复。
- [日语能力评估边界](japanese-capability-assessment.zh-CN.md)——有证据边界的多语言覆盖，不宣称一般流畅度。
- [最终替代验收](final-replacement-acceptance.zh-CN.md)——可重复的 conformance 和无复制边界。
- [Repository Protocol v1](../protocol/v1/specification.zh-CN.md)——规范存储和 receipt contract。

[参考源对齐记录](reference-parity.zh-CN.md)是维护者和审查者使用的比较资料，采用明确的真实性状态；
它不是 adopter 路线，也不是复制实现历史的许可。
