---
author: AI Cockpit maintainers
title: "Android profile start"
description: "Discover an Android quality route without guessing SDK, Gradle, device, signing, or CI facts."
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - adopter_onboarding
---

# Android profile start

After attach, ask the Runtime for a read-only candidate:

```bash
repo=/path/to/android-repository
ai-cockpit profile propose --repo "$repo"
```

A Gradle file or wrapper does not prove the Android SDK, selected module,
variant, emulator/device, signing, credentials, network service, or hosted CI is
ready. Record those facts as Unknown until the project owner confirms them.

Only after approval, confirm and verify the exact repository-owned command. For
a project whose owner approved `./gradlew test`, the bounded local example is:

```bash
ai-cockpit profile confirm --repo "$repo" --program ./gradlew --args test
ai-cockpit verify --repo "$repo" --command ./gradlew --args test --workers 1
```

This does not install Android tooling or claim device, signing, release, or
hosted-CI evidence. Use the [calibration guide](../calibration.md) when any fact
is missing.

[Examples](../README.md) | [中文](android.zh-CN.md) | [日本語](android.ja.md)
