---
author: AI Cockpit maintainers
title: "Android profile 起点"
description: "不猜测 SDK、Gradle、device、signing 或 CI 事实，发现 Android 质量路线。"
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - adopter_onboarding
---

# Android profile 起点

Attach 后让 Runtime 给出只读候选：

```bash
repo=/path/to/android-repository
ai-cockpit profile propose --repo "$repo"
```

Gradle 文件或 wrapper 不能证明 Android SDK、module、variant、emulator/device、signing、
credential、network service 或 hosted CI 已准备好。工程 owner 确认前，这些事实保持 Unknown。

批准后只确认并验证准确的 repository-owned 命令。若 owner 批准 `./gradlew test`：

```bash
ai-cockpit profile confirm --repo "$repo" --program ./gradlew --args test
ai-cockpit verify --repo "$repo" --command ./gradlew --args test --workers 1
```

这不会安装 Android tooling，也不声称已有 device、signing、release 或 hosted-CI 证据。
事实缺失时使用[校准指南](../calibration.zh-CN.md)。

[示例路线](../README.zh-CN.md) | [English](android.md) | [日本語](android.ja.md)
