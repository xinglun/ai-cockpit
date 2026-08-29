---
author: AI Cockpit maintainers
title: "iOS Swift Package fixture 适配"
description: "逐文件记录固定 iOS Swift Package fixture 的 Rust 原生语义映射，不复制安装或 Xcode 实现。"
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

# iOS Swift Package fixture 适配

本页逐一比较固定参考 fixture `examples/fixtures/ios-swift-package/` 的四个文件。
它保留对 Swift Package 对象工程有用的语义，但不承诺 Apple 平台或 Xcode 支持，也不复制
参考源安装器、Make/Python 编排、guard 文件或旧 JSON wire 形状。

## 逐文件映射

| 固定参考源文件 | 源文件事实 | Rust 原生对应与边界 |
| --- | --- | --- |
| `Package.swift` | 使用 Swift tools 5.9，声明 `AppCore` library product，并将 `AppCoreTests` 测试 target 连接到 `AppCore`。 | 将包拓扑作为对象工程/Provider 所有的构建元数据。Work Item 记录相关路径和所有者确认的命令；Runtime 不安装 SwiftPM/Xcode，也不推断 Apple SDK 就绪。 |
| `Sources/AppCore/AppCore.swift` | public `greeting()` 函数稳定返回 `hello`。 | 路径属于对象工程源码 scope。Runtime Contract 校验和证据绑定由对象工程继承，Swift 执行仍由 Provider 负责。 |
| `Tests/AppCoreTests/AppCoreTests.swift` | XCTest case 导入 `AppCore` 并断言 greeting。 | 这是对象工程/Provider 的测试能力。所有者可以确认 `swift test` 或 Xcode scheme；`verify --repo` 记录选定命令和结果。文件本身不能证明 macOS/iOS SDK、模拟器、签名或托管 CI 就绪。 |
| `fixture.json` | 声明 iOS Swift package、Swift Package stack、Swift installer 元数据、macOS 平台以及安全/测试路径。 | Project Profile/Observer 可以记录事实或候选事实。`installerStack` 描述对象工程，不是共享 Runtime 安装；`macos` 是平台标签，不是执行证据。 |

## 安装边界是有意不同的

fixture 的 Swift 元数据不是 AI Cockpit 的安装说明。目标模型是：机器上只安装一份不可变共享
Runtime，然后显式绑定仓库：

```bash
repo=/path/to/swift-repository
ai-cockpit attach --repo "$repo"
ai-cockpit inspect --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

绑定过程只管理仓库自己的 `.ai/`、Contract、Evidence、Knowledge 和 Adapter 状态。它不会复制
Swift fixture、安装 SwiftPM/Xcode、选择 Apple SDK，也不会把工程绑定到 Runtime 全局状态。之后每条
命令都必须带同一个显式 `--repo`；不同对象工程拥有不同的 repository identity 和 evidence chain。

对象工程使用时，应先由所有者和 Provider 确认准确的 `swift test` 或 Xcode 命令。单机结果本身
不是 Provider、Release 或企业证据。

## 继承什么、不继承什么

绑定 Swift 工程继承共享 Runtime 的 Contract 校验、Unknown fail-closed、证据身份、生命周期和
人类 Outcome 规则；不继承 fixture 的 Swift toolchain、Xcode project state、Apple SDK、模拟器、
签名凭据、安装变量，也不代表测试已经执行。scope、profile、snapshot 和 evidence 始终保存在该
工程自己的 repository context 下。

这是语义/文档对等，不是源命令、构建工具或 JSON wire 兼容。真实 iOS/Swift adopter 验收仍应使用
不可变的公开 Runtime 产物单独执行。

[参考索引](README.zh-CN.md) | [English](ios-swift-fixture-adaptation.md) | [日本語](ios-swift-fixture-adaptation.ja.md)
