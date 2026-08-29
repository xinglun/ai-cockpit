---
author: AI Cockpit maintainers
title: "WI-381——参考 parity supersede 决定链接修复"
description: "在不修改历史证据的前提下，将版本化历史替代决定绑定到所有 parity 投影。"
workItemId: WI-381-reference-parity-decision-link-fix
canonical: docs/work-items/WI-381-reference-parity-decision-link-fix.md
audience: [maintainer, reviewer]
status: in_progress
authority: translation
lastVerifiedBy: WI-381-reference-parity-decision-link-fix
terminalArchive: .ai/work-items/archive/WI-381-reference-parity-decision-link-fix.contract.json
terminalVerification: .ai/evidence/WI-381-reference-parity-decision-link-fix.verification.json
terminalFinalization: .ai/decisions/WI-381-reference-parity-decision-link-fix.finalize.json
terminalDecision: .ai/decisions/WI-381-reference-parity-decision-link-fix.close.json
capabilityClaims: [governance_integrity, reference_parity]
---

# WI-381——参考 parity supersede 决定链接修复

[English](WI-381-reference-parity-decision-link-fix.md) · [日本語](WI-381-reference-parity-decision-link-fix.ja.md)

## 意图与边界

已关闭的 WI-379 predecessor 同时拥有 canonical successor 决定和摘要版本化的 supersede 决定。三语 parity 投影必须暴露精确的终态决定路径，治理门才能验证历史恢复链。本 Work Item 只修改文档链接；所有 archive、evidence 和 decision bytes 仍由 Runtime 生成并保持不可变。

## 范围

- 在所有 parity ledger 中添加精确的 WI-379 versioned supersede 决定路径。
- 保持 WI-379 archive、evidence、recovery 和 close 记录不变。
- 保持英文、简体中文、日文投影语义一致。

## 不在范围内

Runtime 代码、生成的 `.ai` 记录、Release artifact 以及全局 Agent/MCP 配置。

## 验收

- 每个 parity 行都引用 WI-379 archive、verification、canonical recovery、versioned supersede recovery 和 superseded close 路径。
- 治理完整性门不再报告 WI-379 `missing_parity_decision`。
- 历史 archive/evidence 摘要保持不变。

## 验证与终态记录

使用带显式 `--repo` 的安装版 Runtime、治理/文档检查和 `cargo test --locked --workspace`。评审合并后，按本页头部记录 archive、verification、finalization 和 close 路径。
