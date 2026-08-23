---
author: AI Cockpit maintainers
title: "Java profile start"
description: "JDK、build tool、module、service、CI facts を推測せず Java quality route を見つける。"
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - adopter_onboarding
---

# Java profile start

Attach 後に read-only candidate を得ます。

```bash
repo=/path/to/java-repository
ai-cockpit profile propose --repo "$repo"
```

Maven/Gradle file だけでは JDK major、wrapper、reactor/module order、profile、private mirror、
service、credential、hosted CI は証明できません。Project command の前に approved working
directory、command、settings path、mirror-access boundary、actual `java` runtime を記録します。

Repository owner が `mvn -B test` を承認した bounded local example です。

```bash
ai-cockpit profile confirm --repo "$repo" --program mvn --args=-B,test
ai-cockpit verify --repo "$repo" --command mvn --args=-B,test --workers 1
```

AI Cockpit は JDK の install/switch、`JAVA_HOME` 変更、private dependency 解決、built-in
Java-major lane gate を行いません。必要 runtime/mirror fact の missing/mismatch は blocked です。

[Calibration](../calibration.ja.md) | [English](java.md) | [中文](java.zh-CN.md)
