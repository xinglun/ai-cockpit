---
author: AI Cockpit maintainers
title: "WI-507——语言适配示例读者边界"
description: "逐个比较 5 个维护中的参考源示例 README，不复制应用技术栈或源治理实现。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
workItemId: WI-507-reference-file-comparison-batch-30
sourceCommit: fde3380f81fea5fd2e288f7a8849f737dc074060
lastVerifiedBy: WI-507-reference-file-comparison-batch-30
terminalArchive: .ai/work-items/archive/WI-507-reference-file-comparison-batch-30.contract.json
terminalVerification: .ai/evidence/WI-507-reference-file-comparison-batch-30.verification.json
terminalFinalization: .ai/decisions/WI-507-reference-file-comparison-batch-30.finalize.json
terminalDecision: .ai/decisions/WI-507-reference-file-comparison-batch-30.close.json
---

# WI-507——语言适配示例读者边界

[English](WI-507-reference-file-comparison-batch-30.md) · [日本語](WI-507-reference-file-comparison-batch-30.ja.md)

## 目标

逐个阅读下一个 5 个维护中的参考源示例 README，并为 Rust Runtime 及其对象工程记录有证据的边界。
本 Work Item 是语义比较，不是复制源示例、安装器、Make bridge、SDK 或应用技术栈的请求。

比对 Runtime：已发布的 `ai-cockpit` v0.2.60，二进制 SHA-256 为
`sha256:f04aa15868a6e3a590b109a7649c37d765cd2bb935213b9cd898f3ddec6b336d`。
参考源基线固定为提交
`fde3380f81fea5fd2e288f7a8849f737dc074060`。

## 文件级决定

固定参考文件被记录为 `reference-only`，因为它们是 provider/应用 onboarding 示例。可移植治理语义仅限于
owner 声明的 scope、verification 命令、evidence 和 repository context；这些语义已经由以下 Rust 原生路线承载。

| 固定参考路径 | 源文件 SHA-256 | Rust 边界 |
| --- | --- | --- |
| `examples/flutter/README.md` | `f9823e1b30e87e2a105869dbdaa03bfac9ed49f73524f9c7bac2326804afe8c7` | `docs/reference/flutter-fixture-adaptation.*`、`docs/reference/contract-fields.md`、`docs/reference/verification-route.md`；不复制 Flutter/Dart 安装器、Make 预设、coverage YAML、应用代码或源 JSON。 |
| `examples/go/README.md` | `ad36fe62949555e0e324c38ad2e6a89f71b6c0f4f4bbc2868973769c8e48dcac` | `docs/getting-started/adopter-configuration.md`、`docs/reference/contract-fields.md`、`docs/reference/verification-route.md`；Go 工具链、Make、coverage 和应用示例仍由对象工程负责。 |
| `examples/java/README.md` | `e83eff645b0f7d21f42590197e88932bf2d106e124c053cff7a12b8470652b4a` | `docs/getting-started/examples/java.md`、`docs/reference/contract-fields.md`、`docs/reference/verification-route.md`；Gradle/Spring/Android 命令和示例代码不是 Runtime 要求。 |
| `examples/kotlin/README.md` | `7324bbf6472865ffc1a0563a3faa1a06d6dffe6be33ec2cc90d794ad197f0e8d` | `docs/getting-started/adopter-configuration.md`、`docs/reference/contract-fields.md`、`docs/reference/verification-route.md`；Kotlin/Gradle 命令仍是对象工程/provider 责任。 |
| `examples/php/README.md` | `a25a87b0b0295677d15da8a5d7751ee3c278cae5946e95020ae2cd79c33dd04b` | `docs/getting-started/adopter-configuration.md`、`docs/reference/contract-fields.md`、`docs/reference/verification-route.md`；不复制 Composer/PHPUnit/PHPStan 命令或应用路径。 |

## 边界与非声明

这些示例不证明 SDK 可用、测试已执行、provider assurance、企业批准或对相应语言技术栈的支持。
对象工程必须在自己的 repository context 下声明 scope、命令、authority 和 evidence。不会继承源 Contract 决定或源 JSON wire shape。

## 验收

- 5 个固定路径均在维护中的本地参考提交上被阅读，并获得非 deferred 的 `reference-only` 台账记录、非空 counterpart 和 reason。
- 英文、简体中文和日文的比较页、parity 页记录相同的 5 个路径、边界和当前计数。
- 不修改参考 checkout、对象/adopter 工程、全局 Agent/MCP 设置或无关 Runtime 行为。
- conformance、文档、Runtime verification、评审 PR、合并、close 和精确清理检查通过。

## 验证

```text
python3 tests/conformance/reference_file_inventory.py --check
bash tests/conformance/reference_file_inventory_test.sh
python3 tests/conformance/reference_inventory_docs_test.py
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
cargo test --locked --workspace
```

本地参考源通过 `AI_COCKPIT_REFERENCE_ROOT` 读取，绝不修改。台账表达的是语义/文档对齐，不是源命令、SDK、provider state 或 JSON wire 兼容。
