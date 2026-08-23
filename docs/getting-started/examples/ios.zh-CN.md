---
author: AI Cockpit maintainers
title: "iOS profile 起点"
description: "不猜测 Xcode、scheme、simulator、signing 或 CI 事实，发现 iOS 质量路线。"
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - adopter_onboarding
---

# iOS profile 起点

Attach 后获取只读候选：

```bash
repo=/path/to/ios-repository
ai-cockpit profile propose --repo "$repo"
```

Xcode project 或 workspace 不能证明 scheme、destination、simulator/device、signing identity、
credential、service 或 hosted macOS CI 已准备好。工程 owner 确认前，每项事实保持 Unknown。

批准后只确认并验证准确命令。以下只是 owner 已批准指定 scheme/simulator 的示意路线：

```bash
ai-cockpit profile confirm --repo "$repo" --program xcodebuild --args=-scheme,App,test
ai-cockpit verify --repo "$repo" --command xcodebuild --args=-scheme,App,test --workers 1
```

所有值都要替换为工程自有事实。这不会安装 Xcode、选择 signing、访问 secrets，或声称已有
device/release/hosted-CI 证据。

[校准](../calibration.zh-CN.md) | [English](ios.md) | [日本語](ios.ja.md)
