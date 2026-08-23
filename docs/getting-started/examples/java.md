---
author: AI Cockpit maintainers
title: "Java profile start"
description: "Discover a Java quality route without guessing JDK, build tool, module, service, or CI facts."
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - adopter_onboarding
---

# Java profile start

After attach, ask for a read-only candidate:

```bash
repo=/path/to/java-repository
ai-cockpit profile propose --repo "$repo"
```

A Maven or Gradle file does not prove the JDK major, wrapper, reactor/module
order, profile, private mirror, service, credentials, or hosted CI. Record the
approved working directory, command, settings path, mirror-access boundary, and
actual `java` runtime before executing a project command.

For a repository whose owner approved `mvn -B test`, a bounded local example is:

```bash
ai-cockpit profile confirm --repo "$repo" --program mvn --args=-B,test
ai-cockpit verify --repo "$repo" --command mvn --args=-B,test --workers 1
```

AI Cockpit does not automatically install or switch a JDK, modify `JAVA_HOME`,
resolve private dependencies, or provide a built-in Java-major lane gate. A
required runtime or mirror fact that is missing or mismatched remains blocked.

[Calibration](../calibration.md) | [中文](java.zh-CN.md) | [日本語](java.ja.md)
