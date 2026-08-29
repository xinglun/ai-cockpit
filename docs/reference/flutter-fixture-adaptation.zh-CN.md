---
author: AI Cockpit maintainers
title: "Flutter fixture 适配"
description: "逐文件记录固定 Flutter fixture 的 Rust 原生语义映射，不复制其安装或 SDK 实现。"
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

# Flutter fixture 适配

本页逐一比较固定参考 fixture `examples/fixtures/flutter-app/` 的四个文件。
它保留对 Flutter 对象工程有用的语义，但不承诺 Flutter SDK 支持，也不复制
参考源的安装器、Make/Python 编排、guard 文件或旧 JSON wire 形状。

## 逐文件映射

| 固定参考源文件 | 源文件事实 | Rust 原生对应与边界 |
| --- | --- | --- |
| `fixture.json` | 声明 Flutter application、Flutter/Dart toolchain、Linux/macOS/Windows 平台以及安全/测试路径。 | Project Profile/Observer 可以记录事实或候选事实。`installerStack` 描述对象工程，不是共享 Runtime 安装契约；平台标签不是执行证据。路径只有在人确认后才进入 Contract scope 和验证输入。 |
| `lib/main.dart` | 简单的 `greeting()` 函数稳定返回 `hello`。 | 将该路径视为对象工程所有的源码。Work Item 记录 intent、scope 和经所有者确认的验证命令；Runtime 不执行或推断 Dart 语义。 |
| `pubspec.yaml` | 声明 fixture 名称和 Dart SDK 范围，未声明包依赖。 | 这是 Observer 可报告的包元数据。SDK 可用性、依赖解析、网络及 lockfile 状态在取得 Provider 证据前保持 Unknown。Runtime 不安装 Flutter 或改写 `pubspec.yaml`。 |
| `test/widget_test.dart` | 使用 `flutter_test` 断言 greeting。 | 这是对象工程/Provider 的测试能力。所有者可以确认 `flutter test`；`verify --repo` 记录结果和身份。文件本身不能证明 Flutter SDK、平台 runner、插件或托管 CI 已就绪。 |

## 安装边界是有意不同的

fixture 的 `installerStack` 和 Dart 元数据不是 AI Cockpit 的安装说明。目标模型是：
机器上只安装一份不可变共享 Runtime，然后显式绑定仓库：

```bash
repo=/path/to/flutter-repository
ai-cockpit attach --repo "$repo"
ai-cockpit inspect --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

绑定过程只管理仓库自己的 `.ai/`、Contract、Evidence、Knowledge 和 Adapter 状态。
它不会复制 Flutter fixture、安装 Flutter/Dart、下载包，也不会把工程绑定到 Runtime
全局状态。之后每条命令都必须带同一个显式 `--repo`；不同对象工程拥有不同的
repository identity 和 evidence chain。

对象工程使用时，应先由所有者和 Provider 确认准确的 Flutter 验证命令。单机
`flutter test` 结果本身不是 Provider、Release 或企业证据。

## 继承什么、不继承什么

绑定 Flutter 工程继承共享 Runtime 的 Contract 校验、Unknown fail-closed、证据身份、
生命周期和人类 Outcome 规则；不继承 fixture 的 SDK、包缓存、平台 runner、安装变量、
Dart 源码，也不代表 Flutter 检查已经执行。scope、profile、snapshot 和 evidence 始终
保存在该工程自己的 repository context 下。

这是语义/文档对齐，不是源命令、构建工具或 JSON wire 兼容。真实 Flutter adopter 验收
仍应使用不可变的公开 Runtime 产物单独执行。

[参考索引](README.zh-CN.md) | [English](flutter-fixture-adaptation.md) | [日本語](flutter-fixture-adaptation.ja.md)
