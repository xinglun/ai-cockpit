---
author: AI Cockpit maintainers
title: "WI-342——参考文档、分发与企业边界"
workItemId: WI-342-reference-documentation-batch-13
description: "逐一比对固定参考源的下一批十个路径，记录有证据的 Rust 对应物，不复制源历史或 wire 格式。"
audience: [maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-342-reference-documentation-batch-13
terminalArchive: .ai/work-items/archive/WI-342-reference-documentation-batch-13.contract.json
terminalVerification: .ai/evidence/WI-342-reference-documentation-batch-13.verification.json
terminalFinalization: .ai/decisions/WI-342-reference-documentation-batch-13.finalize.json
terminalDecision: .ai/decisions/WI-342-reference-documentation-batch-13.close.json
capabilityClaims:
  - reference_parity
---

# WI-342——参考文档、分发与企业边界

## 意图与边界

本 Work Item 在固定参考提交
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` 上逐文件比较 10 个路径。
记录目标工程的语义责任以及有意不同实现或 reference-only 边界；不复制参考源
Python、Make、adopter 记录、provider 声明或 JSON wire 格式。

比较范围包括分发、文档架构与权威、文档 context、企业控制和外部身份。
只更新比较 ledger、三语 parity 文档和本 Work Item 的面向读者记录。
Runtime 行为、发布、adopter 验收、全局 Agent/MCP 配置和后续参考路径均不在范围内。

## 逐文件决定

固定路径及有证据的决定记录在
`tests/conformance/reference_file_inventory.json` 和三语
`docs/reference/reference-file-comparison*` ledger 中。8 个路径为
`implemented-different-by-design`；2 个源专属 control/context 记录为
`reference-only`。没有路径被静默当作 equivalent、deferred 或缺失。

目标继承对象工程边界：一个共享 Runtime、显式 repository context、按 repository
隔离的 `.ai/` 状态、外部 provider evidence，以及不在本地声称企业身份或合规。
Contract/source 原文仍是权威；本地化展示不会改写治理事实。

## 验收

- 每个列出的路径在固定 inventory 中恰好出现一次，具有有证据的分类，以及有效
  的目标对应物或明确的 reference-only 边界。
- 英文、简体中文和日文比较/parity 页面表达相同的语义、非 wire 决定和当前台账计数。
- 不把源计划/context 元数据或源 adopter 控制观察复制到 Runtime，也不当作目标证据。
- inventory、文档和仓库 gate 通过，且不修改生成历史或不可变证据。

[English](WI-342-reference-documentation-batch-13.md) ·
[日本語](WI-342-reference-documentation-batch-13.ja.md)
