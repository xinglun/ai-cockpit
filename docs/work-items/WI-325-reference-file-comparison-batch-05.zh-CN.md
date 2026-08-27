---
author: AI Cockpit maintainers
title: "WI-325——参考源文件对比批次 05"
workItemId: WI-325-reference-file-comparison-batch-05
description: "逐个对比 pinned 参考源的九个文档路径，并登记 Rust 原生语义边界。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-325-reference-file-comparison-batch-05
terminalArchive: .ai/work-items/archive/WI-325-reference-file-comparison-batch-05.contract.json
terminalVerification: .ai/evidence/WI-325-reference-file-comparison-batch-05.verification.json
terminalFinalization: .ai/decisions/WI-325-reference-file-comparison-batch-05.finalize.json
terminalDecision: .ai/decisions/WI-325-reference-file-comparison-batch-05.close.json
---

# WI-325——参考源文件对比批次 05

## 意图与边界

逐个对比 pinned 参考提交
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` 中接下来的九个路径，只保留有证据支持、可由 Rust Runtime 和 adopter 继承的语义，不复制参考源的 Python、Make、fixture 或内部进度实现。

共享 Rust Runtime 仍在工程外部安装，所有 repository 都必须显式绑定
`--repo`。Cursor adopter 反馈作为外部观察输入：稳定 Outcome 和入口门禁已经由现有 Runtime 覆盖；可选的宿主 UI 便利能力不会被本批次默认为已实现。

## 逐文件对比

| pinned 参考路径 | 分类 | Rust/adopter 对应与边界 |
| --- | --- | --- |
| `docs/features/task-outcome-report-self-check.md` | `reference-only` | 当前 Outcome/report/event 页面和 `.ai/README.md`；源 WI22 进度及发布声明属于历史内容，不复制。 |
| `docs/fixtures/real-fixture-evidence.ja.md` | `implemented-different-by-design` | 日文 fixture 布局、Release adopter/upgrade acceptance、distribution 和 adversarial-validation；本地、provider、企业证据保持分离。 |
| `docs/fixtures/real-fixture-evidence.md` | `implemented-different-by-design` | Rust fixture 与不可变 Release adopter/upgrade harness；源七技术栈 `make`/Python 矩阵不是 Runtime 能力。 |
| `docs/guides/lightweight-verification.ja.md` | `implemented-different-by-design` | 日文 verification route、semantics、CI quality、cost 页面；警告不能授权，关键失败会停止。 |
| `docs/guides/lightweight-verification.md` | `implemented-different-by-design` | Rust 阶段验证和动态 light/standard/strict 路由；不复制源 checker 脚本。 |
| `docs/guides/lightweight-verification.zh-CN.md` | `implemented-different-by-design` | 中文 verification route、semantics、CI quality、cost 页面，保持相同 fail-closed 边界。 |
| `docs/installation.md` | `implemented-different-by-design` | reader-first 安装、Release distribution/security 与 `.ai/README.md`；安装不会隐式 attach 或表示已完成 calibration。 |
| `docs/maintainers/adding-or-classifying-a-check.md` | `implemented-different-by-design` | 版本化 gate manifest、动态 route、runner 和回归检查；profile、依赖、skip 与 hard failure 保持显式。 |
| `docs/maintainers/task-outcome-events.md` | `implemented-different-by-design` | 类型化 Rust Task Outcome events、append-only 修正、隐私校验、archive binding 和 human handoff。 |

## 非目标

本 Work Item 不增加 Runtime 命令，不复制源 Python/Make/YAML 或 fixture 文件，不要求
`Makefile.ai`，不改变 Cursor 或全局 Agent/MCP 配置，也不实现可选的
`close-gap`、自动 controls 模板或宿主面板展开。这些是独立产品决定，不隐藏在对比结果中。

## 验收与证据

1. 九个 pinned 路径均已阅读，每个路径恰有一条非空且有证据支持的 inventory 记录。
2. 生成的 inventory 为本 WI 登记八条 `implemented-different-by-design` 和一条
   `reference-only`，没有 deferred 或 migrate gap。
3. 英文、简体中文、日文 parity 页面与本 Work Item 的 source pin、分类和语义边界一致。
4. 内部进度声明、源专属 fixture 结果以及未运行的 provider/enterprise assurance 不得作为当前 Runtime 事实。
5. 安装 Runtime 验证、文档/conformance 检查、hosted CI、生命周期收尾和精确分支/worktree 清理提供终态证据；历史 evidence 不重写。

[English](WI-325-reference-file-comparison-batch-05.md) ·
[日本語](WI-325-reference-file-comparison-batch-05.ja.md)
