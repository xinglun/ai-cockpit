---
author: AI Cockpit maintainers
title: "安装 AI Cockpit"
description: "安装并校验共享 Runtime，不隐式 attach repository。"
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - release_distribution
---

# 安装 AI Cockpit

AI Cockpit 是一份外部共享 Runtime，不是复制到每个工程中的治理目录。按照
[发布与分发](../release/distribution.zh-CN.md)选择不可变公开 Release，下载与准确
target 匹配的制品，并在安装前验证 SHA-256。

单独确认已安装 executable：

```bash
ai-cockpit --version
```

安装本身不会创建 `.ai/`、选择工程质量命令、安装 Agent adapter、证明 hosted CI，
也不会让 repository 自动达到生产可用状态。这些都是独立且需要审查的 repository 操作。

安装后按只读优先路线执行：

```bash
repo=/path/to/repository
ai-cockpit inspect --repo "$repo"
ai-cockpit attach --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

检查所有事实，然后继续[首次校准](first-calibration.zh-CN.md)和
[采用方配置](adopter-configuration.zh-CN.md)。私有镜像或本地 source checkout
不是公开 Release 证据；见[严格安装安全](installation-security.zh-CN.md)。

[快速开始](README.zh-CN.md) | [English](installation.md) | [日本語](installation.ja.md)
