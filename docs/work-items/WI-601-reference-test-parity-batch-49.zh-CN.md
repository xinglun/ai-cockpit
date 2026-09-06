---
title: "WI-601——参考源测试对等批次 49"
description: "逐个比较下一批十个维护中的参考源测试路径，不复制源实现或 wire 格式。"
author: AI Cockpit maintainers
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
workItemId: WI-601-reference-test-parity-batch-49
lastVerifiedBy: WI-601-reference-test-parity-batch-49
terminalArchive: .ai/work-items/archive/WI-601-reference-test-parity-batch-49.contract.json
terminalVerification: .ai/evidence/WI-601-reference-test-parity-batch-49.verification.json
terminalFinalization: .ai/decisions/WI-601-reference-test-parity-batch-49.finalize.json
terminalDecision: .ai/decisions/WI-601-reference-test-parity-batch-49.close.json
---

# WI-601——参考源测试对等批次 49

[English](WI-601-reference-test-parity-batch-49.md) · [日本語](WI-601-reference-test-parity-batch-49.ja.md)

## 意图与边界

逐个重读固定本地参考源中的下一批十个维护测试路径。可移植治理语义映射到 Rust Runtime 或仓库原生门；源/供应商专属夹具、Dependabot 接入和 deprecated-assets 注册表行为保持有界的 `reference-only` 责任。

这是语义对等，不是源命令、Python 模块或 JSON wire 兼容。不修改参考源 checkout、对象工程、全局 Agent/MCP 配置或不可变历史证据。

## 有界结果

十个路径已登记在 `tests/conformance/reference_file_inventory.json` 的 `WI-601-reference-test-parity-batch-49` 批次下：

- 7 项为 `implemented-different-by-design`，由现有类型化 Contract、profile、lifecycle、trust、CI 和文档边界承载。
- 3 项为 `reference-only`：源七技术栈长周期夹具、Dependabot 接入和 deprecated-assets 注册表属于源/供应商边界，不是 Runtime 控制遗漏。

没有发现 `migrate-gap`。三语台账、parity 页面、metadata sidecar、回归脚本和本记录一起更新；追加式台账和源历史不被重写。

## 验收与验证

- 每个选定路径恰有一个分类、对应集合和有界原因。
- 任何确认的可移植遗漏都在本 WI 内修复，不静默延期或隐藏到 successor。
- inventory、回归脚本、metadata、三语比较/parity 页面与本记录一致。
- finish 前通过 conformance、文档、治理完整性和 locked workspace 检查。

下一批对比只能在评审发布、精确清理和可见的人类 Outcome 完成后开始。对象/adopter 工程继承 shared Runtime 及其 repository-bound 隔离；源 Python/Make 模块、provider policy 值、技术栈矩阵和 source wire 不跨越该边界。
