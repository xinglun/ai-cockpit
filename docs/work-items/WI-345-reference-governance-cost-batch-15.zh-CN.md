---
author: AI Cockpit maintainers
title: "WI-345——治理成本与性能文档第 15 批"
workItemId: WI-345-reference-governance-cost-batch-15
description: "逐一比较五个固定治理成本/复杂度/性能文档，记录有界的 Rust 对应物，不虚构源工具。"
audience: [maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-345-reference-governance-cost-batch-15
capabilityClaims:
  - reference_parity
---

# WI-345——治理成本与性能文档第 15 批

## 意图与边界

本 Work Item 逐一比较五个固定参考文档：治理复杂度（英文和日文）、治理成本指标、治理性能预算以及 profile/cost 分离。目标要让 adopter 继承有用的治理边界，但不复制源 Python/Make 维护工具、不虚构耗时证据，也不把成本变成权威。

范围仅包括 inventory、三语 comparison/parity 页面、新的面向读者 reference 页面和本 Work Item。Runtime 代码、源脚本/guard 文件、全局 Agent/MCP 配置、不可变历史和硬性能目标不在范围内。

## 逐文件决定

| 固定参考路径 | 分类 | 有界目标决定 |
| --- | --- | --- |
| `docs/reference/governance-complexity.ja.md` | `reference-only` | 目标文档记录边界并保留不可变 archive/integrity 规则，但不宣称有源 Python/Make scanner、阈值或等价指标。 |
| `docs/reference/governance-complexity.md` | `reference-only` | `inspect`、`status`、`doctor` 与 repository integrity gate 提供目标事实；源复杂度报告仍是不可移植的维护材料。 |
| `docs/reference/governance-cost-metrics.md` | `implemented-different-by-design` | `diagnose` 与 typed verification cost estimate/observation 提供 identity-bound advisory facts。源 JSONL 阶段/等待聚合和报告 wire shape 不是 Rust 要求。 |
| `docs/reference/governance-performance-budget.md` | `implemented-different-by-design` | identity-bound `PerformanceBaseline` sample 与明确 regression budget 拒绝无效/回归测量，不跳过必需验证，也不推导 P95/profile 权威。 |
| `docs/reference/governance-profile-cost-separation.md` | `implemented-different-by-design` | light/standard/strict 路线、operation/stage escalation、`VerificationTier`、`EvidenceAssurance` 和 advisory cost 保持正交。 |

这是语义/文档 parity，不是源命令或 JSON-wire 兼容性。对象工程边界仍是一个共享 Runtime、显式 `--repo`、repository-local evidence、由 policy 拥有的路线要求，并且没有全局 current project。

## 验收与验证

- 五个固定路径在 inventory 中各出现一次，分类如上，且没有 deferred 或 migrate-gap。
- 英文、简体中文、日文 reference/parity 页面表达相同决定和当前台账计数。
- 面向读者的页面说明哪些源细节不可用，不虚构 CLI 命令、profile 决定、指标或 assurance。
- 成本/性能输出明确为 advisory；耗时不能替代 `VerificationTier`、`EvidenceAssurance`、policy 或 protected checks。
- inventory、文档、治理、格式、lint 和锁定 workspace 验证通过。

固定参考提交为 `e5acb677da6621004d96f0ef353c58fe8d3acfbf`，目标基线为 `747cf3d9f846aac52b2a592ec61a874511c18b81`。

[English](WI-345-reference-governance-cost-batch-15.md) ·
[日本語](WI-345-reference-governance-cost-batch-15.ja.md)
