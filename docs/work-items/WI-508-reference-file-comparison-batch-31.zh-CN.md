---
author: AI Cockpit maintainers
title: "WI-508——技术栈适配示例读者边界"
description: "逐个比较 5 个维护中的参考源技术栈示例，不复制源安装器、工具链或应用治理决定。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
workItemId: WI-508-reference-file-comparison-batch-31
sourceCommit: fde3380f81fea5fd2e288f7a8849f737dc074060
lastVerifiedBy: WI-508-reference-file-comparison-batch-31
terminalArchive: .ai/work-items/archive/WI-508-reference-file-comparison-batch-31.contract.json
terminalVerification: .ai/evidence/WI-508-reference-file-comparison-batch-31.verification.json
terminalFinalization: .ai/decisions/WI-508-reference-file-comparison-batch-31.finalize.json
terminalDecision: .ai/decisions/WI-508-reference-file-comparison-batch-31.close.json
---

# WI-508——技术栈适配示例读者边界

[English](WI-508-reference-file-comparison-batch-31.md) · [日本語](WI-508-reference-file-comparison-batch-31.ja.md)

## 目标

逐个阅读下一个 5 个维护中的参考源技术栈适配 README，并为 Rust Runtime 及其对象工程记录有证据的边界。本次是语义比较，不是复制源安装器、Make bridge、SDK、应用示例或 Contract wire shape。

参考源基线固定为提交
`fde3380f81fea5fd2e288f7a8849f737dc074060`。比较和验证使用已安装的公开发布版
`ai-cockpit` v0.2.60，二进制 SHA-256 为
`sha256:f04aa15868a6e3a590b109a7649c37d765cd2bb935213b9cd898f3ddec6b336d`。

## 文件级决定

5 个路径全部标为 `reference-only`：它们是展示技术栈安装、质量命令、coverage 模式以及示例 Contract/Summary 文本的 source/provider onboarding 内容。可移植意义仅限于 owner 声明的 scope、命令、evidence 和 repository context；这些边界已经由现有 Rust 原生路线承载。

| 固定参考路径 | 源文件 SHA-256 | Rust 边界 |
| --- | --- | --- |
| `examples/python/README.md` | `80413e9611a2e03687733d13c433d9377c9cdaafd92b0d4d09b416da9c452d29` | `docs/reference/python-fixture-adaptation.*`、`docs/reference/contract-fields.md`、`docs/reference/verification-route.md`；Python 安装器、Make、coverage 和示例 Contract/Summary 决定仍由对象工程负责。 |
| `examples/ruby/README.md` | `7b8b799edfca2550e63a2493a92e0be98d8ad2a72d30e9b91f381a6aea344f28` | `docs/getting-started/adopter-configuration.md`、`docs/reference/contract-fields.md`、`docs/reference/verification-route.md`；Bundler/RuboCop/RSpec 或 Rake 命令、coverage 和应用示例仍是对象工程/provider 责任。 |
| `examples/rust/README.md` | `60e83d31510f13c79dd5af221608577b50d1d6dfb14e7c0465f8c7f477574149` | `docs/getting-started/adopter-configuration.md`、`docs/reference/contract-fields.md`、`docs/reference/verification-route.md`、`docs/reference/ci-quality-gates.md`；Cargo 命令、内联测试 caveat、Make 预设和示例 Contract/Summary 决定仍由项目自行声明。 |
| `examples/swift/README.md` | `9c5f39905973dfa5400db502750d7eaffe873e287a31d79dc9da691d5e851d6e` | `docs/reference/ios-swift-fixture-adaptation.*`、`docs/reference/contract-fields.md`、`docs/reference/verification-route.md`；SwiftPM/Xcode 命令、coverage、平台/签名假设和示例决定仍由对象工程/provider 负责。 |
| `examples/typescript/README.md` | `036d52e200a13eabb47a7843ccca81b9ecf044aa6e789e51b6bb0af2643fd53f` | `docs/reference/typescript-fixture-adaptation.*`、`docs/reference/contract-fields.md`、`docs/reference/verification-route.md`；npm/Node 脚本、依赖、fixture lifecycle、coverage 和示例决定仍由对象工程/provider 负责。 |

## 边界与非声明

这些示例不证明 SDK 可用、测试已执行、provider 或企业 assurance，也不证明 Runtime 支持相应技术栈。对象工程必须在自己的 repository context 下声明 scope、命令、authority 和 evidence。不会继承源 Contract 决定、源安装器、Make 预设或源 JSON wire shape。

## 验收

- 每个固定路径都在维护中的本地参考提交上被阅读，并拥有带非空 counterpart 和 reason 的非 deferred `reference-only` 台账记录。
- inventory、比较页、parity 页和本 Work Item 记录相同的 5 个决定及当前计数，且没有 `migrate-gap`。
- 不修改参考 checkout、对象/adopter 工程、全局 Agent/MCP 设置或无关 Runtime 行为。
- conformance、文档、Runtime verification、评审 PR、合并、close、发布和精确清理检查通过。

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
