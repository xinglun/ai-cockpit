---
author: AI Cockpit maintainers
title: "Android profile start"
description: "SDK、Gradle、device、signing、CI facts を推測せず Android quality route を見つける。"
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - adopter_onboarding
---

# Android profile start

Attach 後に read-only candidate を得ます。

```bash
repo=/path/to/android-repository
ai-cockpit profile propose --repo "$repo"
```

Gradle file/wrapper があるだけでは Android SDK、module、variant、emulator/device、signing、
credential、network service、hosted CI の readiness は証明できません。Project owner の確認前は
Unknown のままにします。

Approval 後、exact repository-owned command だけを confirm/verify します。Owner が
`./gradlew test` を承認した project の bounded local example です。

```bash
ai-cockpit profile confirm --repo "$repo" --program ./gradlew --args test
ai-cockpit verify --repo "$repo" --command ./gradlew --args test --workers 1
```

Android tooling の install や device、signing、release、hosted-CI evidence の claim は行いません。
Fact が不足する場合は[calibration guide](../calibration.ja.md)を使います。

[Examples](../README.ja.md) | [English](android.md) | [中文](android.zh-CN.md)
