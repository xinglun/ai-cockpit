---
author: AI Cockpit maintainers
title: "WI-482——生命周期、并行与信任层参考源比对"
description: "逐个重读 8 个发生变化的本地参考源路径，并记录 Rust 原生对齐决定。"
audience:
  - maintainer
  - reviewer
workItemId: WI-482-reference-file-comparison-batch-26
status: implemented
authority: canonical
lastVerifiedBy: WI-482-reference-file-comparison-batch-26
terminalArchive: .ai/work-items/archive/WI-482-reference-file-comparison-batch-26.contract.json
terminalVerification: .ai/evidence/WI-482-reference-file-comparison-batch-26.verification.json
terminalFinalization: .ai/decisions/WI-482-reference-file-comparison-batch-26.finalize.json
terminalDecision: .ai/decisions/WI-482-reference-file-comparison-batch-26.close.json
---

# WI-482——生命周期、并行与信任层参考源比对

## 目标

逐个比较本地参考源提交 `fde3380f81fea5fd2e288f7a8849f737dc074060`（上一批之后的变化）中的 8 个文件。
保留有用的治理语义，但不复制参考源 Python Runtime、Make workflow、provider 配置或仅属于源工程的文档布局。

## 边界

本批绑定 Rust 基线 `1f65a3b8bf09e54d4f9600fc5d64d8bbcb3ed62f`，以及已发布的
`ai-cockpit 0.2.57` binary（SHA256 `f03a13251a6fe57783528efbeae6ddd23bc2cc31dd2a1501d5421aac169a1d58`）。
对象/采用方工程、Runtime 新功能和全局 Agent/MCP 配置不在范围内。

## 文件决定

8 个路径全部为 `implemented-different-by-design`：

- 三个 `docs/operations/work-item-lifecycle.*`：由 `docs/reference/agent-workflow.*`、`outcome-report.*` 承载 Rust 原生生命周期、人工暂停和精确清理。
- `docs/reference/agent-parallel-work-items.md`：由 `cross-work-item-dedup.md`、`affected-verification.md`、`agent-workflow.md`、`AGENTS.md` 和 `.ai/README.md` 承载并行边界；对话 handoff 仍由 adapter 负责。
- `docs/reference/ai-cockpit-work-item-lifecycle.md`：由 Rust workflow、Outcome、CI gate 文档和 Runtime 承载；模板专属 pytest 分片和 `REPORT_LANGUAGE` 不属于目标要求。
- 三个 `docs/trust-layer.*`：由 `philosophy.*`、`security/enterprise-governance.*`、`architecture.*`、`capabilities.*` 承载信任链、委托证据、人类决定和限制边界。

源端变化是读者路线/源工作流的收敛，不是 Rust Runtime 缺口。Contract 事实保持 authored language，本地化不能改变治理事实或创建人类决定。

## 验收与验证

- 台账精确记录这 8 个路径，保留 source-change provenance 和此前分类。
- 三语比较文档与 parity 台账逐一列出路径及不复制源实现的边界。
- `python3 tests/conformance/reference_file_inventory.py --check`
- `bash tests/conformance/reference_file_inventory_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `cargo test --locked --workspace`
