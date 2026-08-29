---
author: AI Cockpit maintainers
title: Verification 证据复用 Runtime
description: Rust Runtime 如何在不削弱受保护验证的情况下规划有限的证据复用。
audience: [adopter, maintainer, reviewer]
status: implemented
authority: translation
canonical: docs/reference/verification-evidence-reuse-runtime.md
lastVerifiedBy: WI-379-reference-documentation-batch-18
---

# Verification 证据复用 Runtime

[English](verification-evidence-reuse-runtime.md) · [简体中文](verification-evidence-reuse-runtime.zh-CN.md) · [日本語](verification-evidence-reuse-runtime.ja.md)

AI Cockpit 将规划与执行分开。请求作用域的计划可以把节点标为
`execute` 或 `reuse`，但只有声明的执行路由能运行命令。复用结果是证据，不是跳过
必需门禁的权限。

## 可复用条件

只有当 repository、Work Item、base/head 快照、规范化变更集、命令、scope、stage、
runner/toolchain、policy 和输出身份全部一致时，才可复用通过且未过期的 receipt。
content、diff、environment 是现有验证节点的绑定维度，不会生成第二套 checker API。
缺失、格式错误、过期、外部或矛盾 receipt 都是 `unknown`，必需节点会重新执行。

scope、安全/信任、治理、coverage、identity、source-bound、supply-chain 等受保护
门禁，只要 policy 或 stage 要求就必须执行。`stage_not_applicable` 不是执行证据。

## 可审计事实与边界

结果记录 planned、executed、reused、stale-rerun、unknown-rerun、protected-node、
耗时、worker 和 receipt identity。降低调用次数必须来自真实 adapter call-count 观测，
不能由耗时或缓存标签推断。即使物理执行共享，每个 Work Item 仍获得自己的身份绑定
receipt。

这是 Rust-native 语义边界；参考源的 Python 模块、Make target 和 JSON wire shape 不
复制到 Runtime 或 adopter。
