---
author: AI Cockpit maintainers
title: 测试削弱信号
description: Rust Runtime 中基于 snapshot 检测验证强度下降。
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: translation
canonical: docs/reference/test-weakening-guard.md
lastVerifiedBy: WI-378-reference-documentation-batch-17
capabilityClaims:
  - test_weakening_detection
---

# 测试削弱信号

[English](test-weakening-guard.md) · [简体中文](test-weakening-guard.zh-CN.md) · [日本語](test-weakening-guard.ja.md)

Rust Runtime 在 preflight 和 Contract quality gate 中，根据声明的 base 与当前 repository snapshot 推导测试和 coverage weakening 信号。Agent 的文字不是证据，Signal 为空也不证明完整语义覆盖。

## Signal 边界

检测器观察仓库相对路径的 tracked 变化，例如删除测试、新增 skip/disable 标记、删除负向/安全回归、把必需检查改为非阻塞、降低 coverage 要求和明确的成功绕过。无效 revision、路径 traversal、非常规文件、越界 symlink、不可读/二进制输入会被保守处理为 unknown 或 blocked，不会标绿。

`test_weakening` 是阻断性的治理 Signal。Coverage weakening 默认是 review/unknown，除非适用 Contract 或 policy 将其设为阻断。动态 quality route 根据变更面选择适当分析强度；strict/release route 可以要求完整检查。

每个非 continue 结果都带有稳定 finding 和恢复条件。恢复验证强度，或提供可独立评审的需求变更证据，然后针对同一 base 重跑。环境变量、本地 receipt 或人工文字都不能绕过 critical signal。Provider 端必需检查以及动态/生成测试语义仍属于外部事实或明确限制。

这是参考源 Test Weakening Guard 的 Rust-native 语义对应，不发布源 Python 模块、Make target 或源 JSON wire format。
