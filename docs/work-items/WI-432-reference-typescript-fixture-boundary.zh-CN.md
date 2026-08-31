---
author: AI Cockpit maintainers
title: "WI-432——TypeScript web fixture 边界"
workItemId: WI-432-reference-typescript-fixture-boundary
description: "逐文件比较固定 TypeScript web fixture，记录 Rust-native 的 reference-only 边界，不复制 Node 工具链。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-432-reference-typescript-fixture-boundary
terminalArchive: .ai/work-items/archive/WI-432-reference-typescript-fixture-boundary.contract.json
terminalVerification: .ai/evidence/WI-432-reference-typescript-fixture-boundary.verification.json
terminalFinalization: .ai/decisions/WI-432-reference-typescript-fixture-boundary.finalize.7ed22daac35a32d6f53289562f5fc955ba076854ff0483799f42c54a7a199eed.json
terminalDecision: .ai/decisions/WI-432-reference-typescript-fixture-boundary.close.json
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
---

# WI-432——TypeScript web fixture 边界

## 意图与边界

在固定参考提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 中逐一读取
`examples/fixtures/typescript-web/` 的十一个文件。它们是参考工程的
TypeScript/npm 可执行样例，不是 Rust Runtime 代码、Node/TypeScript toolchain
支持、可移植治理策略或 provider/企业证据。

目标台账将每个路径标记为 `reference-only`，并在[适配说明](../reference/typescript-fixture-adaptation.zh-CN.md)
和[逐文件比较台账](../reference/reference-file-comparison.zh-CN.md)中说明 Rust-native 对象工程边界。
不复制源 fixture、npm 依赖、安装器或 Node 生命周期脚本。

## 验收

- 十一个固定路径逐一读取，并在机器台账中各出现一次。
- 每个路径都有非空 reason 与 counterpart，分类为 `reference-only`，本批不留下
  `deferred-next-batch` 或 `migrate-gap`。
- 英文、简体中文、日文的适配、比较、索引和 parity 路线在 source pin、文件列表和不复制边界上保持一致。
- 台账与文档门通过，且不改变 Runtime 治理语义、对象工程工具链或全局 Agent/MCP 配置。

## 验证与非声明

本批是语义/参考边界对齐，不是 TypeScript toolchain 支持、源命令兼容、JSON wire 兼容或第二技术栈
adopter 验收。逐文件事实以机器台账为准。

[English](WI-432-reference-typescript-fixture-boundary.md) · [日本語](WI-432-reference-typescript-fixture-boundary.ja.md)
