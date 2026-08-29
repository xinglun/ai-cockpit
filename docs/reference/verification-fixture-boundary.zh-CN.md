---
author: AI Cockpit maintainers
title: Verification fixture 边界
description: 隔离 repository fixture 的内容以及它不能证明的事项。
audience: [contributor, maintainer, reviewer]
status: implemented
authority: translation
canonical: docs/reference/verification-fixture-boundary.md
lastVerifiedBy: WI-379-reference-documentation-batch-18
---

# Verification fixture 边界

[English](verification-fixture-boundary.md) · [简体中文](verification-fixture-boundary.zh-CN.md) · [日本語](verification-fixture-boundary.ja.md)

Repository 测试可以使用临时副本运行 Rust Runtime。fixture 只包含源代码和
repository-local Protocol 输入，不包含调用方 Runtime 状态。除非测试显式声明需要，
否则排除 Git metadata、worktree、虚拟环境、Cargo/build 输出以及语言和工具缓存。

保留的 Work Item 历史不会为了缩小 fixture 而复制，fixture 初始化也不会删除源 checkout。
fixture 结果只是本地测试证据，不是 provider、托管 CI、adopter、生产或企业证据。Release
和 adopter 声明必须使用各自不可变 artifact 与隔离 receipt。
