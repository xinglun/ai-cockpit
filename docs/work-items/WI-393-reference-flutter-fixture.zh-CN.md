---
workItemId: WI-393-reference-flutter-fixture
title: "参考 Flutter fixture 适配"
author: AI Cockpit maintainers
description: "逐文件记录固定 Flutter fixture 的语义映射，并明确共享 Runtime 安装边界。"
type: documentation
audience:
  - adopter
  - contributor
  - maintainer
authority: canonical
status: in_progress
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
lastVerifiedBy: WI-393-reference-flutter-fixture
---

# WI-393：参考 Flutter fixture 适配

## Intent

逐一比较四个固定 Flutter fixture 文件，记录 Rust 原生/对象工程映射，
不复制 Flutter 安装或 SDK 实现。

## Scope

- `examples/fixtures/flutter-app/fixture.json`
- `examples/fixtures/flutter-app/lib/main.dart`
- `examples/fixtures/flutter-app/pubspec.yaml`
- `examples/fixtures/flutter-app/test/widget_test.dart`
- 三语 Flutter 适配、参考比较、parity、索引和 Work Item 记录

## Acceptance

1. 每个源文件都有独立语义映射或明确的有界不适用结论。
2. 文档说明 Flutter/Dart 检查由对象工程/Provider 负责；缺少 SDK、依赖、网络、
   平台、插件和 CI 事实时保持 Unknown。
3. 安装说明是一份在对象工程外的不可变共享 Runtime 加显式 `attach --repo`；
   不复制源安装器、构建文件或 wire 产物。
4. 清单、parity、链接和三语记录绑定 `e5acb677`。
5. 安装 Runtime 能验证文档和 conformance 检查。

## Evidence 边界

本 WI 只证明语义/文档对等，不证明 Flutter 工具链支持或发布后 Flutter adopter 验收。
