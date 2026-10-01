---
author: AI Cockpit maintainers
workItemId: WI-1051-mac-only-prerelease-v1-0-1-rc-1
title: v1.0.1-rc.1 macOS arm64 预发布
description: 发布一个可选的 macOS arm64 预发布候选，用于试用 WI-1050 Contract amendment 准入修复，并保持 v1.0.0 为稳定版。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1051-mac-only-prerelease-v1-0-1-rc-1
---

[English](WI-1051-mac-only-prerelease-v1-0-1-rc-1.md) · [日本語](WI-1051-mac-only-prerelease-v1-0-1-rc-1.ja.md)

# WI-1051——v1.0.1-rc.1 macOS arm64 预发布

本 Work Item 准备并发布一个可选的 `v1.0.1-rc.1` 预发布候选，用于在
macOS arm64 上试用 WI-1050 Contract amendment 准入修复。它不是稳定版；
`v1.0.0` 仍是最新稳定版和安装基线。

## 边界

- 候选源码包含 WI-1050 修复提交
  `09e7eead64bebacda98d71e1907c01e6008a8f88`（tree
  `3c2ac29a38ea1eb8d8e1c856424834761ab16630`）。
- 只发布 macOS arm64 可执行文件及其匹配的 SHA-256 sidecar。
- 不发布非 macOS 资产、不改变稳定版指针、不修改无关 tag/release，也不包含 Task9 变更。
- Runtime 正式验证、hosted CI 和非 macOS 验证属于独立证据边界；本地构建不能代替它们。

## macOS 本地证据

已独立完成以下构建：

```text
cargo build --locked --release --package cockpit-cli --target aarch64-apple-darwin
Finished `release` profile [optimized] target(s) in 1m 31s
```

隔离 prefix 中的副本报告 `ai-cockpit 1.0.1-rc.1`，`--help` 返回 exit 0，
文件类型为 arm64 Mach-O，且字节与构建产物相同。生成的 sidecar 已通过
`shasum -a 256 -c`：

```text
SHA-256: 7e15777b24480dc88c880698876a0cff03aeb5dfe019a6a68db3ecfbe9409b7f
大小：10086320 bytes
```

这些是独立的本地构建和启动观察，不是 Runtime verification receipt。当前
Runtime Summary 仍没有正式 verification evidence；Contract 的八个场景在
Runtime 中仍标记为未验证。

## 发布与验收边界

预发布说明必须披露尚未验证的 Runtime review/verification、hosted CI、
非 macOS 目标及回归检查。发布后需核对精确 provider release，下载可执行文件
和 checksum 并验证下载字节，同时确认 `v1.0.0` 仍是最新稳定版。剩余证据和
生命周期决策解决前，本 Work Item 保持进行中。

若需要撤回，只能在确认 provider 身份后移除精确的 `v1.0.1-rc.1` 预发布、
对应资产及其 tag。不得修改 `v1.0.0` 或无关 release/tag；修正版必须使用
新的预发布版本号。
