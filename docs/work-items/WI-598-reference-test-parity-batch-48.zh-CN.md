---
title: "WI-598——参考源测试对等批次 48"
description: "逐个比较下一批二十个维护中的参考源测试路径，不复制源实现或 wire 格式。"
author: AI Cockpit maintainers
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
workItemId: WI-598-reference-test-parity-batch-48
lastVerifiedBy: WI-598-reference-test-parity-batch-48
terminalArchive: .ai/work-items/archive/WI-598-reference-test-parity-batch-48.contract.json
terminalVerification: .ai/evidence/WI-598-reference-test-parity-batch-48.verification.json
terminalFinalization: .ai/decisions/WI-598-reference-test-parity-batch-48.finalize.json
terminalDecision: .ai/decisions/WI-598-reference-test-parity-batch-48.close.json
---

# WI-598——参考源测试对等批次 48

[English](WI-598-reference-test-parity-batch-48.md) · [日本語](WI-598-reference-test-parity-batch-48.ja.md)

## 意图与边界

逐个比较固定本地参考源中的下一批二十个维护文件。可移植的治理语义
映射到 Rust Runtime 或仓库原生门；技术栈工具链和源测试夹具保留为
`reference-only`。

这是语义对等，不是源命令、Python 模块或 JSON wire 兼容。不修改对象工程、
全局 Agent/MCP 配置或不可变历史证据。

## 有界结果

二十个路径已登记在
`tests/conformance/reference_file_inventory.json` 的
`WI-598-reference-test-parity-batch-48` 批次下：

- 18 项为 `implemented-different-by-design`，由类型化 Git、仓库、profile、
  evidence、CI 与 release 边界承载。
- 2 项为 `reference-only`：Java Runtime 选择和 Bandit 基线属于供应商/工具链，
  不是 Runtime 要求。

没有发现 `migrate-gap`。三语台账与 metadata sidecar 一起更新；台账追加式保留，
不重写源历史。

## 验收与验证

- 每个路径恰有一个分类、对应集合和有界原因。
- 任何确认的可移植遗漏都在本 WI 内修复，不静默延期。
- inventory、回归脚本、metadata、三语比较/对等页与本记录一致。
- finish 前通过 conformance、文档、治理完整性和锁定 workspace 检查。

本批次之后的发布必须使用不可变公开制品和 adopter/N-1 验收脚本。只有评审发布、
精确清理和可见的人类 Outcome 完成后，才能开始下一批对比。
