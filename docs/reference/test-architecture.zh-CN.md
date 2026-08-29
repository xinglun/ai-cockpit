---
author: AI Cockpit maintainers
title: 测试架构
description: Rust 工程的分层、负向优先验证与质量门归属。
audience:
  - contributor
  - maintainer
  - reviewer
status: current
authority: translation
canonical: docs/reference/test-architecture.md
lastVerifiedBy: WI-378-reference-documentation-batch-17
capabilityClaims:
  - layered_verification
---

# 测试架构

[English](test-architecture.md) · [简体中文](test-architecture.zh-CN.md) · [日本語](test-architecture.ja.md)

验证采用分层、负向优先方式。只有仓库中存在明确证据时，某层才会报告为 verified；不可用的层报告为 `not_applicable` 或 `unknown`，不会被静默标绿。

| 层 | Rust 证据边界 |
| --- | --- |
| Protocol/schema/state machine | `cargo test --workspace`、typed protocol tests、lifecycle 和 property 回归 |
| Repository transaction 与 lifecycle | attach、Contract、checkpoint、verify、finish、archive、close、recovery、isolation 的 repository/CLI 集成测试 |
| Verification executor | 有界 argv 执行、worker 限制、复用 identity、失败保留和 scope 测试 |
| Security/adversarial | conformance 与荒诞案例 fixture、path/symlink/identity 篡改、prompt-injection 与 weakening 回归 |
| Hosted platform | GitHub Actions Windows/runtime 与 V1 semantic oracle；Provider 状态仍是外部证据 |
| Release/adopter | 不可变公开归档、checksum/SBOM/provenance、新 adopter 与 N-1 upgrade harness |
| Documentation/governance | 三语 metadata、parity、inventory、status-promotion 和 governance-integrity 检查 |

动态 quality route 根据变更面、Contract policy 和 stage 选择 `light`、`standard` 或 `strict`。这表示 Verification 强度，不表示 Evidence Assurance。低成本 route 可以省略不相关层，但不能降低必需 floor，也不能把 unknown 变成 pass。

本地检查只证明仓库事实，不能证明 Provider approval、enterprise identity、完整外部消费者兼容或普遍测试覆盖。`target/` 下报告和生成 receipt 是证据输出，不得手工编辑。
