---
author: AI Cockpit maintainers
title: "iOS profile start"
description: "Xcode、scheme、simulator、signing、CI facts を推測せず iOS quality route を見つける。"
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - adopter_onboarding
---

# iOS profile start

Attach 後に read-only candidate を得ます。

```bash
repo=/path/to/ios-repository
ai-cockpit profile propose --repo "$repo"
```

Xcode project/workspace だけでは scheme、destination、simulator/device、signing identity、
credential、service、hosted macOS CI の readiness は証明できません。Project owner が確認するまで
各 fact を Unknown に保ちます。

Approval 後に exact command だけを confirm/verify します。Owner-approved scheme/simulator を
持つ project の illustrative bounded route です。

```bash
ai-cockpit profile confirm --repo "$repo" --program xcodebuild --args=-scheme,App,test
ai-cockpit verify --repo "$repo" --command xcodebuild --args=-scheme,App,test --workers 1
```

すべての値を project-owned facts に置き換えます。Xcode install、signing selection、secret access、
device/release/hosted-CI evidence の claim は行いません。

[Calibration](../calibration.ja.md) | [English](ios.md) | [中文](ios.zh-CN.md)
