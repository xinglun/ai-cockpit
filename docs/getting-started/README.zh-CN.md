---
author: AI Cockpit maintainers
title: "快速开始"
description: "安全安装共享 Runtime 并 attach 第一个 repository。"
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - adopter_onboarding
---

# 快速开始

新 adopter repository 使用以下路线：

1. 按[发布与分发](../release/distribution.zh-CN.md)安装不可变公开 Release 并验证 digest。
2. 运行 `ai-cockpit inspect --repo /path/to/repository`，再运行 `ai-cockpit attach --repo /path/to/repository`。
3. 运行 `ai-cockpit status --repo /path/to/repository` 与 `ai-cockpit doctor --repo /path/to/repository`。
4. 只有在需要时才安装 Agent adapter；`attach` 不修改 Agent 文件或全局 MCP 配置。
5. 用 `ai-cockpit work-item new --repo /path/to/repository --id <id> --mode code` 创建 `not_ready` 骨架。
6. 检查只读 profile candidate，并完成[首次校准](first-calibration.zh-CN.md)。
7. 完成[采用方配置](adopter-configuration.zh-CN.md)中的外部审查、安全与 CI 决定。
8. 运行完整 Runtime 原生[首个 Work Item](first-work-item.zh-CN.md)。

安装是共享 Runtime 操作；repository attach 是显式操作，会创建 repository-local `.ai/`。
一份 Runtime 可以服务多个 repository，但不会共享它们的 Work Item、evidence 或 active context。

## 读者路线

- [30 秒开始](30-second-start.zh-CN.md)——inspect、attach、status 与 doctor。
- [安装](installation.zh-CN.md)——不可变公开 Runtime 与 repository 分离。
- [Repository profile 校准](calibration.zh-CN.md)——确认一条工程自有质量命令。
- [标准采用指南](standard-adoption-guide.zh-CN.md)——完整采用顺序。
- [安全与 Release 验证](security-release-verification.zh-CN.md)——供应链与外部证据边界。
- 示例：[Android](examples/android.zh-CN.md)、[iOS](examples/ios.zh-CN.md)、[Java](examples/java.zh-CN.md)。

完成首个 Work Item 后继续阅读[功能](../features/README.zh-CN.md)和
[运维](../operations/README.zh-CN.md)。

[文档首页](../README.zh-CN.md) | [English](README.md) | [日本語](README.ja.md)
