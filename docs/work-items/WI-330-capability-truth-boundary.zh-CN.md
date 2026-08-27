---
author: AI Cockpit maintainers
title: "WI-330——能力真相边界决定"
workItemId: WI-330-capability-truth-boundary
description: "在不复制 V1 资产的前提下，完成参考源能力声明、freshness 与真相矩阵文档的逐文件语义对比。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-330-capability-truth-boundary
terminalArchive: .ai/work-items/archive/WI-330-capability-truth-boundary.contract.json
terminalVerification: .ai/evidence/WI-330-capability-truth-boundary.verification.json
terminalFinalization: .ai/decisions/WI-330-capability-truth-boundary.finalize.0c1ecf840859c3ce2fda21da34d25e8e742386d4d8de7674ade851d217dcdcdc.json
terminalDecision: .ai/decisions/WI-330-capability-truth-boundary.close.json
capabilityClaims:
  - reference_parity
---

# WI-330——能力真相边界决定

## 意图与边界

本 Work Item 按固定源提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 重新读取四个
参考文件并闭合语义对比。目标 Rust Runtime 仍是 repository governance layer，不复制源
Python checker、矩阵字节或 V1 runtime state。

## 逐文件决定

| 固定源路径 | 分类 | 目标责任 |
| --- | --- | --- |
| `docs/reference/capability-claim-authoring.md` | `reference-only` | 目标文档 metadata 只是描述。`capability show` 与 capability registry 报告观察到的绑定事实，不通过 lexical trigger 授权公开措辞。 |
| `docs/reference/capability-evidence-freshness.md` | `reference-only` | Work Item verification receipt 有 identity/freshness 检查，但源 Capability Truth 行过期和 portable-environment 策略不是当前 Runtime 功能。 |
| `docs/reference/capability-truth-matrix.json` | `reference-only` | 源三十行矩阵不是 Rust wire format 或授权源。目标 capability truth 是按请求生成、绑定 repository/snapshot 的 projection，并明确 adopter 与外部排除。 |
| `docs/reference/capability-truth-matrix.md` | `reference-only` | 目标能力/采用页面说明观察事实、repository evidence、adopter 安装、delegated provider evidence 和企业边界，不宣传源 matrix/checker。 |

这些是明确的产品边界，不是未登记遗漏。未来若实现 claim binding 或行级 freshness，必须由
单独的人工拥有 Work Item 定义 Rust-native schema、证据生成、过期处理、三语 scope 和 adopter
验收。

## 验收

1. inventory 与三语 comparison 页面为四个固定路径分别记录分类、对应与原因。
2. 三语 comparison 与 parity 页面陈述一致的“不复制、非授权”边界；现有 capability
   索引不在本次文档范围内。
3. 不新增源 Python script、源 matrix JSON、V1 state、全局 Agent/MCP 配置或无证据能力声明。
4. inventory/文档门、Runtime 验证、reviewed PR、合并、finalization、close 和精确清理全部通过。

[English](WI-330-capability-truth-boundary.md) · [日本語](WI-330-capability-truth-boundary.ja.md)
