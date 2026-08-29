---
author: AI Cockpit maintainers
title: "Android fixture 适配"
description: "逐文件记录固定参考 Android fixture 的 Rust 原生映射，不复制安装器或构建实现。"
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
capabilityClaims:
  - semantic_reference_mapping
lastVerifiedBy: documentation-acceptance
---

# Android fixture 适配

本页逐一比较固定参考 fixture `examples/fixtures/android-app/` 的四个文件。
它保留 Android adopter 可用的语义，但不承诺 Android 工具链支持，也不复制参考源的
installer、Make/Python 编排、guard 文件或旧 JSON wire 结构。

## 逐文件映射

| 固定源文件 | 源事实 | Rust 原生对应与边界 |
| --- | --- | --- |
| `app/src/main/kotlin/example/MainActivity.kt` | 小型 Kotlin `greeting()` 函数返回稳定值。 | 将路径作为 repository-owned source，在 Work Item `scope`/`outOfScope` 中明确决定并验证 owner 批准的命令；Runtime 不执行或推断 Kotlin 语义。 |
| `app/src/test/kotlin/example/MainActivityTest.kt` | `kotlin.test` 断言 greeting。 | 这是 adopter/provider 的测试能力。工程 owner 可确认 `./gradlew test` 等命令，再由 `verify --repo` 记录结果和 identity。仅凭此文件不能证明 SDK、emulator、signing 或 hosted CI 已准备好。 |
| `fixture.json` | 声明 `projectType`、`stack`、`installerStack`、toolchain、platforms 以及安全/测试路径。 | Project Profile/Observer 可以把它们记录为事实或候选事实。`installerStack` 描述 adopter，不是共享 Runtime 的安装契约；platform 名称也不是执行证据。安全/测试路径只有在人工确认后才能成为 Contract scope 与 verification 输入。 |
| `settings.gradle.kts` | 配置 Gradle 仓库、根项目名和 `:app` inclusion。 | 这只是构建拓扑证据。Gradle/Android 依赖下载、SDK/device readiness、credential、network 和 CI 在 owner 批准的 provider 检查提供证据前保持 Unknown。 |

## 安装语义有意不同

参考 fixture 是示例工程，其 stack 元数据不是 AI Cockpit 的安装方式。目标模型是：每个 adopter
之外只安装一份不可变共享 Runtime，然后显式 attach 到仓库：

```bash
repo=/path/to/android-repository
ai-cockpit attach --repo "$repo"
ai-cockpit inspect --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

attach 负责该仓库自己的 `.ai/`、Contract、evidence、knowledge 与 adapter 状态；不会复制 Android
fixture，不会安装 Gradle 或 Android SDK，也不会把工程绑定到 Runtime 的全局状态。后续每个命令都必须带
同一个显式 `--repo`；不同 adopter 具有独立的 repository identity 和 evidence chain。

adopter 路线请参见 [Android profile 起点](../getting-started/examples/android.zh-CN.md)：先提出只读候选，
由 owner 确认准确的 Gradle 命令，再执行验证。本地结果不是 provider、release 或 enterprise 证据。

## 对象工程继承什么

已 attach 的 Android 工程继承共享 Runtime 的 Contract 校验、Unknown 时 fail-closed、证据 identity、
生命周期和面向人的 Outcome 规则；不继承参考 fixture 的安装变量、Gradle 文件、Kotlin 源码，也不继承
“Android 检查已经执行”的声明。每个工程在自己的 repository context 下保留独立 scope、profile、snapshot
和 evidence。

这是语义/文档对等，不是源命令、构建工具或 JSON wire 兼容。真正的 Android adopter 验收仍是独立的发布后
测试，并且必须使用不可变的公开 Runtime 制品。

[参考索引](README.zh-CN.md) | [English](android-fixture-adaptation.md) | [日本語](android-fixture-adaptation.ja.md)
