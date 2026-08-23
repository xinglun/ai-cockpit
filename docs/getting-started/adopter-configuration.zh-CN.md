---
author: AI Cockpit maintainers
title: "采用方配置"
description: "采用前必须由 repository owner 决定的审查、安全、恢复、profile 与 CI 配置。"
audience:
  - adopter
  - security
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - adopter_onboarding
---

# 采用方配置

AI Cockpit 提供 repository-local 治理机制，但不会替采用方选择人员、provider identity、
安全联系人或组织 policy。请在独立且经过审查的 Work Item 中完成：

- 保护探测到的 remote default branch，并启用 repository 批准的审查 policy；
- 配置 CODEOWNERS 或等价 provider rule，使用真实 owner；
- 在 `SECURITY.md` 中公布私密漏洞报告路线、支持版本、响应预期与披露 policy；
- 指定恢复和事件 owner，以及安全 stop/resume 路线；
- 确认工程质量命令及其 coverage 边界；
- 让 hosted CI 运行 repository-owned gates，并保留 provider 证据；Work Item 记录中不得放 secrets；
- 记录哪些 identity、approval、signing、provenance 与 retention 声明仍属外部责任。

使用 Runtime facts 检查 repository，但不要把它们当成 provider 证明：

```bash
repo=/path/to/repository
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
ai-cockpit agent doctor --repo "$repo" --json
```

本地结果为 green 不代表 branch protection 或 required review 已启用。外部证据缺失或矛盾时
保持 Unknown，由负责人员或 provider 解决。

[标准采用指南](standard-adoption-guide.zh-CN.md) | [English](adopter-configuration.md) | [日本語](adopter-configuration.ja.md)
