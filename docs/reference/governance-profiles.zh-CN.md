---
author: AI Cockpit 维护者
title: 治理配置级别
description: 面向 Light、Standard、Strict Work Item 的风险质量路由。
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: translation
canonical: docs/reference/governance-profiles.md
lastVerifiedBy: WI-346-reference-governance-profiles-status
capabilityClaims:
  - risk_based_quality_routing
---

# 治理配置级别

[English](governance-profiles.md) · [简体中文](governance-profiles.zh-CN.md) · [日本語](governance-profiles.ja.md)

AI Cockpit 根据仓库事实、Work Item Contract、执行阶段和适用策略选择质量路线。
路线按风险递增：`light < standard < strict`。混合变更采用最高适用路线，未知或空的路径证据不能降低路线。

这里说明的是验证强度，不是 assurance（保证等级），也不能替代人的授权。

## 三种配置级别

| 级别 | 典型变更 | 目标路线 |
| --- | --- | --- |
| `light` | 文档、注释、不可执行示例、仅格式变更 | 聚焦质量检查 |
| `standard` | 普通源码、测试、缺陷修复和小型重构 | 项目验证加引用影响检查 |
| `strict` | 治理、CI、安装器、安全、依赖、破坏性/公共 API、迁移、校准或证据 Schema 变更 | 完整仓库和供应链检查 |

`release` 是操作类别，不是第四种配置级别。涉及发布资源的操作可以在
strict 底线之上增加 release-preflight、制品、校验和、SBOM、provenance 和 adopter 检查。
普通 strict 变更不会因为级别名称而自动获得发布路线。

## 配置效果与 assurance

必须保持以下维度正交：

- `VerificationTier`（`T0`–`T3`）表示需要多强的验证。
- `EvidenceAssurance`（`SelfDeclared`、`RepositoryVerified`、
  `ProviderVerified`、`EnterpriseVerified`）表示谁或什么能够为证据背书。
- 成本和复用观测只描述资源使用情况，是 advisory，不能降低要求，也不能把 unknown 变成 green。

`T3` 不等于 `ProviderVerified`，`strict` 也不等于 `EnterpriseVerified`。
Tier 或 assurance 要求必须能追溯到 Organization Policy、Project Policy、Release Policy、受保护 Gate
或人工拥有的 Contract。Planner 可以提出升级，但不能把策略藏在计划内部。

所有路线都保留相同的强制控制底线：scope、trust、lifecycle 和 evidence integrity。
可选的 heavy 或成本检查不是授权或安全开关。未知配置、损坏策略、危险路径、无效 base、
不完整 override 或删除强制控制都会 fail closed。

## 路线如何选择

仓库绑定的路线在受保护命令执行前评估：

```text
仓库 snapshot + Contract + 阶段/策略
                 ↓
        `ai-cockpit gate --repo <path> --contract <file>`
                 ↓
      声明的验证命令 / Hosted Gate
```

路线会相对于 Contract base 检查已提交、已暂存、未暂存和未跟踪路径。
生成的 receipt 绑定仓库、Work Item、base/snapshot、选择的配置级别、Verification Tier、
assurance 要求、理由和 Gate 身份。Receipt 是路由证据，不是授权令牌。

显式指定的级别只能提升自动结果，不能降低它。降级需要有期限且限定当前 Work Item 的 human override，
并包含批准证据、理由、已知风险和未运行的检查；不能形成永久例外。

## 会话与仓库边界

质量报告写入者使用工作树本地的非阻塞锁。同一工作树的第二次调用会 fail closed；不同工作树可以并行。
共享 Runtime 没有 current project 或全局 active Work Item。每个 adopter 仓库都必须显式传递 `--repo`，
自己的 Contract、证据和 adapter 记录保持隔离。

参考模板中的 `make ai-cockpit-quality` 和 Python router 是比对资料，不是本 Rust 仓库必须复制的命令。
目标支持的边界是已安装 Runtime、显式仓库上下文、类型化 Contract/verification 记录和仓库声明的 CI Gate。
本地结果不会被静默提升为 Hosted 或 enterprise assurance。

## 如何安全阅读结果

使用 `ai-cockpit work-item outcome --repo <path> --id <work-item>` 阅读面向人的交接结果。
绿色表示可以审阅列出的证据，不表示已授权 merge、release、发布或安全声明；黄色表示证据或决定不完整；
红色表示必需控制失败或上下文无效，必须停止。参阅[如何阅读 Cockpit 状态](how-to-read-cockpit-status.zh-CN.md)。

