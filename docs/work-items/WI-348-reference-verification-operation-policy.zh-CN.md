---
author: AI Cockpit 维护者
title: "WI-348——验证、操作时策略与 provider 边界参考批次"
workItemId: WI-348-reference-verification-operation-policy
description: "逐一比较固定的十个参考路径，并补齐有界的 Rust 验证/策略差异。"
audience: [maintainer, reviewer]
status: implemented
authority: translation
canonical: docs/work-items/WI-348-reference-verification-operation-policy.md
lastVerifiedBy: WI-348-reference-verification-operation-policy
terminalArchive: .ai/work-items/archive/WI-348-reference-verification-operation-policy.contract.json
terminalVerification: .ai/evidence/WI-348-reference-verification-operation-policy.verification.json
terminalFinalization: .ai/decisions/WI-348-reference-verification-operation-policy.finalize.json
terminalDecision: .ai/decisions/WI-348-reference-verification-operation-policy.close.json
capabilityClaims: [reference_parity, operation_time_policy_evaluation]
---

# WI-348——验证、操作时策略与 provider 边界参考批次

[English](WI-348-reference-verification-operation-policy.md) · [日本語](WI-348-reference-verification-operation-policy.ja.md)

## 意图与边界

逐一比较固定提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 的后续十个路径。
在 Rust 中保留验证、多语言、性能和操作时治理语义，但不复制源 Python/Make
Runtime、生成评估字节、provider 全局配置或历史 provider 事实。

共享外部 Runtime 仍是 request-scoped。每个 adopter/对象工程使用显式 `--repo`；
Contract、证据、性能事实和决定保存在各自仓库。

## 逐文件决定

| 参考路径 | 分类 | 决定 |
| --- | --- | --- |
| `docs/reference/japanese-capability-assessment.md` | implemented-different-by-design | 将源矩阵映射到有限的三语读者/Outcome/荒诞/安装/文档检查，不宣称一般流畅度。 |
| `docs/reference/lightweight-verification-and-soft-gates.md` | implemented-different-by-design | 文档化按比例路线、内容绑定复用、partial 依赖处理、单调升级和可见 advisory 边界。 |
| `docs/reference/multilingual-semantic-parity.md` | implemented-different-by-design | 三语保持 Runtime 自有展示事实一致；Contract 值保留编写语言。 |
| `docs/reference/open-pr-issue-reconciliation-662.json` | reference-only | 历史源/provider 清单，不是当前 GitHub 或发布事实。 |
| `docs/reference/open-pr-issue-reconciliation-662.md` | reference-only | 历史对账叙述，不是当前授权。 |
| `docs/reference/operation-time-policy-reevaluation.{ja,md,zh-CN}.md` | implemented-different-by-design | 增加 Rust Core `OperationTimeRequest` 严格 fail-closed 评估器；只评估，不执行或授予 provider 权限。 |
| `docs/reference/performance-diagnosis.md` | implemented-different-by-design | 将源诊断映射到 request-scoped Rust `diagnose` 和 advisory cost observation，不臆造 provider 等待/P95/assurance。 |
| `docs/reference/pre-release-documentation-alignment.json` | reference-only | 历史生成评估收据；目标文档使用自己的检查和证据。 |

## 验证

- 台账包含正好十条 WI-348 记录：七条 `implemented-different-by-design` 和三条
  `reference-only`，没有 deferred 或 migrate-gap。
- `OperationTimeRequest` 对不支持的 schema、未知操作、操作/目标/范围不一致、
  缺失范围/权限、过期证据、不可信输入和未分类影响 fail-closed，且从不执行操作。
- 英语、简体中文、日语文档链接完整；固定展示标签本地化但不改变 Contract 字节。
- 比较台账记录固定的目标基线和当前计数；历史 provider/pre-release 记录不复制到
  `.ai/` 或 status。
- Rust、文档/台账、格式、lint 和 locked workspace 验证通过；审查合并前必须有
  安装 Runtime 生成的可见人工 Outcome。
