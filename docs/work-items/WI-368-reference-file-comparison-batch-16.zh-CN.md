---
author: AI Cockpit maintainers
title: "WI-368——参考源逐文件比对批次 16"
description: "逐一比对固定参考源的 11 个文件，并明确 Rust 原生边界。"
workItemId: WI-368-reference-file-comparison-batch-16
audience: [maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-368-reference-file-comparison-batch-16
terminalArchive: .ai/work-items/archive/WI-368-reference-file-comparison-batch-16.contract.json
terminalVerification: .ai/evidence/WI-368-reference-file-comparison-batch-16.verification.json
terminalFinalization: .ai/decisions/WI-368-reference-file-comparison-batch-16.finalize.json
terminalDecision: .ai/decisions/WI-368-reference-file-comparison-batch-16.close.json
capabilityClaims:
  - reference_parity
---

# WI-368——参考源逐文件比对批次 16

[English](WI-368-reference-file-comparison-batch-16.md) · [日本語](WI-368-reference-file-comparison-batch-16.ja.md)

## 意图与边界

本 Work Item 针对固定参考提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf`
逐文件比较接下来的 11 个路径，并记录它们由 Rust Runtime 承接、明确外部负责，
还是仅保留为历史参考。

目标仍是共享的一份已安装 Runtime 与显式 `--repo` 上下文。参考源的 Python/Make/YAML
编排、生成历史、provider 全局配置、source JSON wire 兼容和公开发布不在范围内。
语义映射不表示命令或字段完全相同。

## 逐文件决定

| 固定参考路径 | 分类 | 有界目标决定 |
| --- | --- | --- |
| `docs/reference/pre-release-documentation-alignment.md` | `reference-only` | 历史生成的对齐证据；目标文档由自身仓库 gate 检查。 |
| `docs/reference/pre-release-documentation-review.json` | `reference-only` | 历史五策略审查记录；源状态和发现不能授权目标发布。 |
| `docs/reference/project-test-timing-baseline.json` | `implemented-different-by-design` | 映射到身份绑定的 Rust 性能样本和 advisory budget；耗时不能降低验证要求。 |
| `docs/reference/provider-backed-governance-validation.md` | `implemented-different-by-design` | provider 配置、分支保护、reviewer 身份和 hosted 控制保持为委托证据。 |
| `docs/reference/real-absurd-injection-cases.md` | `implemented-different-by-design` | 通过 canonical manifest 和 Rust 测试保留 15 个语义 cases 与 12 个命名 RAI cases。 |
| `docs/reference/real-absurd-injection-cases.zh-CN.md` | `implemented-different-by-design` | 保留中文语义边界；源 prose 不是 Runtime authority。 |
| `docs/reference/real-absurd-injection-cases.ja.md` | `implemented-different-by-design` | 保留日文语义边界，不宣称一般语言流畅度。 |
| `docs/reference/real-adopter-reference-validation.md` | `implemented-different-by-design` | 使用不可变公开 Release 的 adopter/upgrade 验收，隔离仓库、Runtime、生命周期和清理证据。 |
| `docs/reference/reference-impact-gate.md` | `reference-only` | 源静态 scanner/schema/Make 命令未提供；操作时策略只是更窄的已声明事实边界。 |
| `docs/reference/reference-impact-gate.zh-CN.md` | `reference-only` | 同样明确有界 gap，不导入源 scanner 或 provider 声明。 |
| `docs/reference/reference-impact-gate.ja.md` | `reference-only` | 同样明确有界 gap，不导入源 scanner 或 provider 声明。 |

参考源的 reference-impact 页面暴露了目标 Standard profile 的过度声明。
本批次已在三语 profile 文档中修正：Standard 需要显式声明 impact evidence，
不暗示静态 caller、dynamic reference、external consumer 或 monitoring scanner。
现有 operation-time evaluator 仍可评估已声明的 operation/target/scope/authority/
freshness/trust/impact facts，但不能替代该 scanner。

参考源的 real-absurdity 三语页面对命名场景数量不一致。本目标以 canonical manifest
（15 个结构化 wording cases、12 个命名 RAI cases）为机器事实，并保留差异，不自行猜测。

## 验收与验证

- 每个固定路径在 inventory 中恰好出现一次，有非空理由，且本批没有 deferred 或 migrate-gap。
- 历史与 provider 记录保持非权威；耗时/成本保持 advisory；adopter 证据绑定不可变公开 Release 和隔离仓库。
- 三语 adversarial route 的确定性语义一致，并明确源文件计数差异。
- 三语 Standard profile 不再过度声明 reference-impact scanner，并链接操作时策略限制。
- inventory、文档 metadata/links、parity、governance integrity 和定向测试通过；不新增源 Python/Make/V1 文件或全局 Agent/MCP 配置。
- 使用已安装 v0.2.38 Runtime 执行仓库绑定生命周期，在 merge/close/清理前显式交付人类 Outcome。

固定参考提交：`e5acb677da6621004d96f0ef353c58fe8d3acfbf`。
