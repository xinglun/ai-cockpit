---
author: AI Cockpit maintainers
title: WI-638——参考源重新基线第 54 批
description: 逐一复核固定参考源的下一批 60 条路径，不复制源实现。
audience: [maintainer, reviewer, adopter]
workItemId: WI-638-reference-rebaseline-batch-54
status: implemented
authority: human:repository-owner
lastVerifiedBy: WI-638-reference-rebaseline-batch-54
terminalArchive: .ai/work-items/archive/WI-638-reference-rebaseline-batch-54.contract.json
terminalVerification: .ai/evidence/WI-638-reference-rebaseline-batch-54.verification.json
terminalFinalization: .ai/decisions/WI-638-reference-rebaseline-batch-54.finalize.json
terminalDecision: .ai/decisions/WI-638-reference-rebaseline-batch-54.close.json
---

# WI-638——参考源重新基线第 54 批

本 Work Item 在固定本地参考提交
`a9224aed77b5c317b53c4551a9eec306d91ee330` 上逐个复核下一批 60 条非历史路径。
参考源只作为规格语料；Python、Shell、Make、provider、fixture 以及源 JSON
字节不复制到 Rust 工程，也不写入对象工程。

逐文件的机器台账是
`tests/conformance/reference_file_inventory.json`，其中绑定源路径、前序决定、
Rust 对应物、分类和理由。语义对比允许 Rust 使用不同的命令、类型化记录、测试、
发布边界或读者文档，只要保留可移植责任而不导入源的本地 authority。

本批结果为：55 条 `implemented-different-by-design`，由 Rust Runtime、仓库原生
测试、CI/release/adopter 边界或读者文档承载；5 条 `reference-only`，即源跨 Work
Item 汇总、废弃资产/理解度记录、安装计划 wizard 测试和 Java fixture 测试，保留为
源/provider 或 fixture 证据而不是 Runtime authority。没有 `deferred-next-batch`
或 `migrate-gap`。

本批固定的 60 条路径与英文记录中的列表完全一致；完整 counterpart、前序分类和边界
理由以机器台账为准。对象/adopter 工程继承同一 shared Runtime、显式 repository
context、隔离 Contract/evidence/knowledge、动态验证、fail-closed 生命周期和可见
human Outcome；不继承源 Python/Make 实现或源 wire 格式。

## 验收

验证前必须通过固定 inventory、文档、parity 和仓库 quality gate。只有在本 Work Item
合并、关闭、post-close 文档晋级完成并清理精确 branch/worktree 后，才能开始下一批。
本轮六个批次全部完成后才统一发布版本；本 Work Item 不隐含中间发布。

参见：[English](WI-638-reference-rebaseline-batch-54.md) ·
[日本語](WI-638-reference-rebaseline-batch-54.ja.md)。
