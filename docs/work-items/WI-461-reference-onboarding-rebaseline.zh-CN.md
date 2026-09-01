---
author: AI Cockpit maintainers
title: "WI-461——getting-started onboarding 重新基线"
workItemId: WI-461-reference-onboarding-rebaseline
description: "重新阅读本地参考源中发生变更的九个入门页面，并完成逐文件语义台账。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-461-reference-onboarding-rebaseline
terminalArchive: .ai/work-items/archive/WI-461-reference-onboarding-rebaseline.contract.json
terminalVerification: .ai/evidence/WI-461-reference-onboarding-rebaseline.verification.json
terminalFinalization: .ai/decisions/WI-461-reference-onboarding-rebaseline.finalize.json
terminalDecision: .ai/decisions/WI-461-reference-onboarding-rebaseline.close.json
---

# WI-461——getting-started onboarding 重新基线

本 Work Item 重新阅读维护者本地参考源中发生变更的九个入门页面。比较的
历史提交为 `e5acb677da6621004d96f0ef353c58fe8d3acfbf`，当前固定提交为
`fde3380f81fea5fd2e288f7a8849f737dc074060`，路径为
`/Users/sei-rinn/dev/workspace_python/ai-cockpit-template`。不访问公开参考仓库，
也不复制源实现。

[English](WI-461-reference-onboarding-rebaseline.md) · [日本語](WI-461-reference-onboarding-rebaseline.ja.md)

## 逐文件决定

| 固定参考路径 | 分类 | Rust 对应物与边界 |
| --- | --- | --- |
| `docs/getting-started/first-work-item.md` | `implemented-different-by-design` | Rust 页面保留 repository-bound 的 start → preflight → checkpoint → verify → finish → archive → reviewed merge → cleanup → close 全流程、可见人类 Outcome 和人工 review 停止点；源 Make 命令及已删除的 `REPORT_LANGUAGE` 参数不复制。 |
| `docs/getting-started/first-work-item.zh-CN.md` | `implemented-different-by-design` | 中文页面保留相同生命周期与停止条件，并要求显式 `--repo`；语言展示不会改变 Contract 事实。 |
| `docs/getting-started/first-work-item.ja.md` | `implemented-different-by-design` | 日文页面保留相同生命周期、provider-resource 边界和精确清理路径，本批修正了重复的 merge 段落。 |
| `docs/getting-started/security-release-verification.md` | `implemented-different-by-design` | Rust release/distribution 与 installation-security 页面通过当前 manifest/SHA256SUMS 路径保留 tag、digest、SBOM、provenance、provider 责任和 adopter 隔离边界；不复制源 `release.json` 投影。 |
| `docs/getting-started/security-release-verification.zh-CN.md` | `implemented-different-by-design` | 中文发布路线保留证据分离和不一致时 fail-closed 规则，并使用 Rust 原生发布资产与外部 provider 边界。 |
| `docs/getting-started/security-release-verification.ja.md` | `implemented-different-by-design` | 日文发布路线保留 digest、provenance、SBOM 和公开 adopter 限制，不导入源安装器行为。 |
| `docs/getting-started/standard-adoption-guide.md` | `implemented-different-by-design` | Rust 指南保留 reader-first 的 install、attach、calibration、adapter、Work Item、Outcome、merge、cleanup、close 阶段，并使用共享 Runtime；源 Make 工作流字节不是目标 Contract。 |
| `docs/getting-started/standard-adoption-guide.zh-CN.md` | `implemented-different-by-design` | 中文指南保留有序 adoption 边界和显式仓库所有权，并使用 Rust CLI 路径。 |
| `docs/getting-started/standard-adoption-guide.ja.md` | `implemented-different-by-design` | 日文指南保留相同的有序 adoption 路径与共享 Runtime 边界，不复制源专用命令。 |

本批是语义/文档对等，不是源文件或 JSON wire 对等。目标明确使用一份共享安装
Runtime 和显式 `--repo`；对象工程继承治理边界，同时保持仓库状态和 provider 配置隔离。

## 验证边界

台账保留 `sourceChangedSincePrevious`、`previousBatch`、`previousClassification` 等
不可变的比对溯源信息，同时将九条记录从 `deferred-next-batch` 提升为有证据的结果。
本批必须通过离线源策略、inventory 检查、三语 getting-started 语义检查、文档验收、
governance-integrity 检查和声明的 locked Rust 验证命令。本批仅修改比对台账与文档，
不修改 Runtime 行为或对象/采用方工程。
