---
author: AI Cockpit maintainers
title: "WI-336——前五个治理文档路径"
workItemId: WI-336-reference-docs-foundation
description: "逐个比较前五个延期的参考源治理文档，记录 Rust-native 边界，不复制源工具。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-336-reference-docs-foundation
---

# WI-336——前五个治理文档路径

## 意图与边界

在固定参考提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 上逐个比较五个延期路径。
目标是为 adopter 建立可审计的 Rust-native 映射，不是逐字复制参考源的 Python、Make、provider
或历史内容。

## 文件级比较

| 固定参考源路径 | 分类 | Rust/adopter 对应物与边界 |
| --- | --- | --- |
| `docs/reference/cross-wi-integration.md` | `reference-only` | 每个 Work Item 的 archive 校验、`reference-parity` 和面向人的 Outcome 构成目标审计边界。源 WI-04..WI-13 聚合报告及 UI receipt 不是 Runtime 命令。 |
| `docs/reference/dependabot-intake.md` | `not-applicable` | Dependabot bot 分支接入是 provider 专属能力。通用 delegated evidence 和显式 Work Item source binding 仍由外部/provider 负责。 |
| `docs/reference/deprecated-assets-registry.json` | `reference-only` | 显式 Runtime lifecycle、不可变历史和精确 resource finalization 负责清理边界；不提供源 registry 或 Make 扫描。 |
| `docs/reference/deprecated-assets.md` | `reference-only` | registry hygiene 和过时命令链说明属于源文档；Rust 不声称存在 `check-deprecated-assets`。 |
| `docs/reference/derived-artifacts.md` | `implemented-different-by-design` | Runtime typed Contract/evidence/archive 事实与 status/Outcome projection 分离，并在 Outcome/verification 文档中说明；derived view 不能授权后续决定。 |

## 非目标

本 Work Item 不增加跨 Work Item 报告引擎、Dependabot 集成、过时资产删除命令、derived-artifact
registry，也不引入源 Python/Make/V1 实现。不改写不可变历史，不修改全局 Agent/MCP 配置。

## 验收与验证

1. 五个固定路径各有一条台账记录，并包含明确分类、对应物和不夸大的理由。
2. English、简体中文和日本語 comparison/parity 台账对分类一致，并说明语义/非 wire 边界。
3. 文档说明现有 Rust 事实/视图与外部 provider 边界，不把源命令宣称为可用。
4. Inventory、文档、parity 和 locked workspace 验证全部通过。

[English](WI-336-reference-docs-foundation.md) ·
[日本語](WI-336-reference-docs-foundation.ja.md)
