---
author: AI Cockpit maintainers
title: "Java profile 起点"
description: "不猜测 JDK、build tool、module、service 或 CI 事实，发现 Java 质量路线。"
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - adopter_onboarding
---

# Java profile 起点

Attach 后获取只读候选：

```bash
repo=/path/to/java-repository
ai-cockpit profile propose --repo "$repo"
```

Maven 或 Gradle 文件不能证明 JDK major、wrapper、reactor/module 顺序、profile、private mirror、
service、credential 或 hosted CI。执行工程命令前，记录已批准的工作目录、命令、settings path、
mirror access 边界与实际 `java` runtime。

若 repository owner 批准 `mvn -B test`，有边界的本地示例如下：

```bash
ai-cockpit profile confirm --repo "$repo" --program mvn --args=-B,test
ai-cockpit verify --repo "$repo" --command mvn --args=-B,test --workers 1
```

AI Cockpit 不会自动安装或切换 JDK、修改 `JAVA_HOME`、解析私有依赖，也没有内置 Java-major
lane gate。必要 runtime 或 mirror 事实缺失/不匹配时保持 blocked。

[校准](../calibration.zh-CN.md) | [English](java.md) | [日本語](java.ja.md)
