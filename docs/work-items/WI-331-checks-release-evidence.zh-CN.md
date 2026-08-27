---
author: AI Cockpit maintainers
title: "WI-331——检查目录与 CI/发布证据"
workItemId: WI-331-checks-release-evidence
description: "比对固定版本的检查与 CI/发布证据文档，记录 Rust 原生责任边界。"
audience:
  - adopter
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-331-checks-release-evidence
capabilityClaims:
  - reference_parity
---

# WI-331——检查目录与 CI/发布证据

## 意图与边界

本 Work Item 按提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 逐一比对以下两个固定参考路径：

| 固定源路径 | 目标责任 |
| --- | --- |
| `docs/reference/checks-catalog.md` | `docs/reference/checks-catalog.*` 目录描述 Runtime、workspace、conformance 和发布检查，不复制源 Make/Python 执行。 |
| `docs/reference/ci-release-evidence.md` | `docs/reference/ci-release-evidence.*`、版本化门禁清单、CI/Release workflow 与 adopter harness 描述 provider 派生证据及其所有权。 |

目标仍是共享的外部 Rust Runtime、仓库本地 `.ai/` 状态和显式 `--repo` 上下文。这是语义责任
对齐，不是源命令、wire 或字节对齐。本地检查、托管 provider 证据、公开 Release 证据和企业
assurance 保持分离。

## 验收

1. inventory 为两个固定路径分别记录明确分类、目标对应和有证据的原因。
2. 英文、简体中文和日文页面描述相同的检查层次、profile 选择、CI 证据、Release 证据及失败边界。
3. 文档区分验证覆盖强度与 Evidence Assurance，不把本地或 staged 结果提升为 provider 或企业证明。
4. 不复制源 Makefile、Python/V1 执行器或 provider 全局配置，也不手动修改生成的生命周期真相。
5. inventory 与文档回归通过且没有 `migrate-gap`；Runtime 验证、reviewed PR、合并、finalization、
   close 及精确分支/worktree 清理提供终态证据。

[English](WI-331-checks-release-evidence.md) · [日本語](WI-331-checks-release-evidence.ja.md)
