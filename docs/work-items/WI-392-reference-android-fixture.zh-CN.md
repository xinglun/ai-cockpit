---
workItemId: WI-392-reference-android-fixture
title: "参考 Android fixture 适配"
author: AI Cockpit maintainers
description: "逐文件映射固定 Android fixture 的语义，并明确共享 Runtime 安装边界。"
type: documentation
audience:
  - adopter
  - contributor
  - maintainer
authority: canonical
status: implemented
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
lastVerifiedBy: WI-392-reference-android-fixture
terminalArchive: .ai/work-items/archive/WI-392-reference-android-fixture.contract.json
terminalVerification: .ai/evidence/WI-392-reference-android-fixture.verification.json
terminalFinalization: .ai/decisions/WI-392-reference-android-fixture.finalize.53b26b80706cab70f1fb4c8c3772cbf92475c25fa11d5141c906ccafa9566fea.json
terminalDecision: .ai/decisions/WI-392-reference-android-fixture.close.json
---

# WI-392：参考 Android fixture 适配

## 意图

逐一比较固定参考 Android fixture 的四个文件，记录 Rust 原生/adopter 映射，不硬抄 Android 安装或构建实现。

## 范围

- `examples/fixtures/android-app/app/src/main/kotlin/example/MainActivity.kt`
- `examples/fixtures/android-app/app/src/test/kotlin/example/MainActivityTest.kt`
- `examples/fixtures/android-app/fixture.json`
- `examples/fixtures/android-app/settings.gradle.kts`
- 三语 Android 适配、参考对比、parity、索引和 Work Item 记录

## 验收

1. 每个源文件都有独立语义映射，或明确有界的不适用说明。
2. 指南说明 Android/Gradle 检查由 adopter/provider 负责；缺失的 SDK、device、signing、network 和 CI 事实保持 Unknown。
3. 安装说明采用 adopter 外部的一份不可变共享 Runtime 加显式 `attach --repo`；不复制源 installer/build/wire 制品。
4. 清单、parity、链接和三语记录都绑定 `e5acb677`。
5. 使用已安装 Runtime 验证文档和 conformance 检查。

## 证据边界

本 Work Item 只证明语义/文档对等，不证明 Android 工具链支持或发布后 Android adopter 验收。
