---
author: AI Cockpit maintainers
title: "iOS profile start"
description: "Discover an iOS quality route without guessing Xcode, scheme, simulator, signing, or CI facts."
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - adopter_onboarding
---

# iOS profile start

After attach, ask for a read-only candidate:

```bash
repo=/path/to/ios-repository
ai-cockpit profile propose --repo "$repo"
```

An Xcode project or workspace does not prove the selected scheme, destination,
simulator/device, signing identity, credentials, service, or hosted macOS CI.
Keep each fact Unknown until the project owner confirms it.

Only after approval, confirm and verify the exact command. For a project whose
owner approved one scheme and simulator, an illustrative bounded route is:

```bash
ai-cockpit profile confirm --repo "$repo" --program xcodebuild --args=-scheme,App,test
ai-cockpit verify --repo "$repo" --command xcodebuild --args=-scheme,App,test --workers 1
```

Replace every value with project-owned facts. This does not install Xcode,
select signing, access secrets, or claim device/release/hosted-CI evidence.

[Calibration](../calibration.md) | [中文](ios.zh-CN.md) | [日本語](ios.ja.md)
