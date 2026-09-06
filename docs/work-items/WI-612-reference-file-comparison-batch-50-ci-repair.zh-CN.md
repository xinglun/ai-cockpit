---
title: "WI-612——参考源测试对等批次 50"
description: "逐个比较下一批二十个维护中的参考源测试路径，不复制源实现或 wire 格式。"
author: AI Cockpit maintainers
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
workItemId: WI-612-reference-file-comparison-batch-50-ci-repair
lastVerifiedBy: WI-612-reference-file-comparison-batch-50-ci-repair
terminalArchive: .ai/work-items/archive/WI-612-reference-file-comparison-batch-50-ci-repair.contract.json
terminalVerification: .ai/evidence/WI-612-reference-file-comparison-batch-50-ci-repair.verification.json
---

# WI-612——参考源测试对等批次 50

[English](WI-612-reference-file-comparison-batch-50-ci-repair.md) · [日本語](WI-612-reference-file-comparison-batch-50-ci-repair.ja.md)

## 意图与边界

逐个重读固定本地参考源中的下一批二十个维护测试路径。可移植治理责任映射到
Rust Runtime、原生测试或文档；源/供应商夹具和参与者研究材料保持有界的
`reference-only`。这是语义对等，不是源命令、Python 模块或 JSON wire 兼容。

## 有界结果与验收

完整路径映射记录在 `tests/conformance/reference_file_inventory.json` 和三语比较
台账中。15 项为 `implemented-different-by-design`，5 项为 `reference-only`；未发现
可移植实现遗漏或 `migrate-gap`。metadata、文档、parity、inventory 检查和
`cargo test --locked --workspace` 必须通过。对象/adopter 工程继承 shared Runtime
及 repository-bound 隔离，不继承源 Python/Make、provider policy 值、技术栈 preset 或
source wire。
