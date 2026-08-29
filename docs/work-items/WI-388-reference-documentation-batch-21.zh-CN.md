---
author: AI Cockpit maintainers
title: "WI-388——参考文档第 21 批"
workItemId: WI-388-reference-documentation-batch-21
description: "比较六个固定的排查、采用稳定性和威胁模型文档，在不复制源 authority 的前提下记录有界 Rust-native parity。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-388-reference-documentation-batch-21
terminalArchive: .ai/work-items/archive/WI-388-reference-documentation-batch-21.contract.json
terminalVerification: .ai/evidence/WI-388-reference-documentation-batch-21.verification.json
terminalFinalization: .ai/decisions/WI-388-reference-documentation-batch-21.finalize.796631a3301dfcc04a7ef0e0381c01f3d8fca7bffbf9278763ea588a53bbc5d4.json
terminalDecision: .ai/decisions/WI-388-reference-documentation-batch-21.close.json
---

# WI-388——参考文档第 21 批

## 意图与边界

在参考提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 上逐一阅读下面六个路径。通过当前 Rust-native 文档路线保留面向读者的治理含义，同时不把源命令、provider authority 或历史稳定性结论带入目标。

| 固定参考路径 | 分类 | Rust 对应/有界决定 |
| --- | --- | --- |
| `docs/security/threat-model.md` | implemented-different-by-design | 三语 `docs/security/threat-model.*` 保留资产、信任边界、fail-closed 威胁和外部控制限制；不声称能识别所有恶意意图或认证企业安全。 |
| `docs/template-adopter-stability-matrix.md` | implemented-different-by-design | `docs/reference/final-replacement-acceptance.md`、`docs/getting-started/standard-adoption-guide.md`、`docs/reference/ci-release-evidence.md` 与 adopter harness 分布承载证据类型和采用边界；模板单独证据不是外部稳定性证明。 |
| `docs/troubleshooting.md` | implemented-different-by-design | 三语 `docs/reference/troubleshooting.*` 提供停止状态、恢复和证据保留，而不是仅兼容性跳转页。 |
| `docs/troubleshooting/installation.ja.md` | implemented-different-by-design | 日语安装、严格验证和排查页面保留不确定即停止与显式 attach。 |
| `docs/troubleshooting/installation.md` | implemented-different-by-design | 英语安装、严格验证和排查页面保留不确定即停止、不可变制品检查和显式 attach。 |
| `docs/troubleshooting/installation.zh-CN.md` | implemented-different-by-design | 中文安装、严格验证和排查页面保留相同的恢复与 repository context 边界。 |

## 验收

- 每个固定文件均已阅读，并有明确 inventory 分类与对应关系。
- 三语 comparison、parity 和 Work Item 记录一致；`migrate-gap` 保持为零。
- 不复制或升级源 Python/Make 命令、provider authority 或历史证据。
- 明确共享 Runtime 与对象/adopter 继承边界：一份已安装 binary、显式 `--repo`、隔离的 repository facts 与 evidence。
- 文档、inventory、治理和已安装 Runtime 验证检查通过。

## 验证与非声明

这是语义/文档 parity，不是源命令、JSON-wire 或 provider 状态兼容。目标可以把一项责任分布到多个读者路线；当有界对应关系和非声明已记录时，不存在同名文件不等于遗漏。

[English](WI-388-reference-documentation-batch-21.md) · [日本語](WI-388-reference-documentation-batch-21.ja.md)
