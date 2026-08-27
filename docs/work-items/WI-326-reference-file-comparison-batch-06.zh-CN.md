---
author: AI Cockpit maintainers
title: "WI-326——参考源文件对比批次 06"
workItemId: WI-326-reference-file-comparison-batch-06
description: "逐个比对固定参考源的九个质量门、总览、设计思想与关闭计划路径，并登记有证据支持的 Rust 原生边界。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-326-reference-file-comparison-batch-06
---

# WI-326——参考源文件对比批次 06

## 意图与边界

逐个比对固定参考提交
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` 中的以下 9 个路径。为对象工程保留
读者和治理相关语义，不复制源 Python、Make、installer、fixture 或内部进度实现。

共享 Rust Runtime 仍在工程外部安装，所有 repository 请求都必须显式携带
`--repo`。本批是文档与 conformance 台账工作，不增加 Runtime 行为，也不宣称 source
wire 兼容。

## 逐文件对比

| pinned 参考路径 | 分类 | Rust/adopter 对应与边界 |
| --- | --- | --- |
| `docs/non-make-adaptation.ja.md` | `implemented-different-by-design` | 安装与 Agent workflow 路线表达外部 Runtime 和仓库本地 adapter 边界。对象工程自己的技术栈命令仍在 Core 之外；不复制也不要求源 `Makefile.ai` 桥接层。 |
| `docs/operations/quality-gates.ja.md` | `implemented-different-by-design` | 日文 CI 质量门与 manifest 路线保留门禁所有权、证据、追踪及按策略选择 `light`/`standard`/`strict` 的动态路由。不复制源 Make/Python 编排。 |
| `docs/operations/quality-gates.md` | `implemented-different-by-design` | 版本化 Rust 原生门禁清单与 CI 路由保留质量门语义，同时让托管 CI 与对象工程技术栈检查各归其责任边界。 |
| `docs/operations/quality-gates.zh-CN.md` | `implemented-different-by-design` | 中文质量门与 manifest 路线保留同样的证据和动态路由边界；源 Make/Python checker 注册表不是目标命令。 |
| `docs/overview.ja.md` | `implemented-different-by-design` | Rust architecture、capabilities、Agent workflow 与 command 路线保留源五层总览，并以 request-scoped、repository-bound 方式治理；不复制源 status/verification registry。 |
| `docs/philosophy/design-philosophy.ja.md` | `implemented-different-by-design` | 日文产品边界、能力和企业治理文档保留校准信任、证据优先于自我声明、与风险相称的控制以及人的责任。 |
| `docs/philosophy/design-philosophy.md` | `implemented-different-by-design` | 英文产品边界、能力和企业治理文档保留同样原则；Core 不是 Agent Runtime、安全沙箱、身份提供方或合规证书。 |
| `docs/philosophy/design-philosophy.zh-CN.md` | `implemented-different-by-design` | 中文产品边界、能力和企业治理文档保留同样原则与明确非目标。 |
| `docs/plans/harden-work-item-pr-closure.md` | `reference-only` | 源文件是 Python `ai-finish`/`ai-close` 的内部历史强化计划。当前 Rust lifecycle 与 governance-integrity 路线保留关闭意图，但过时的实现步骤和命令名不是当前 Runtime 能力。 |

## 非目标

本 Work Item 不增加 Runtime 命令，不复制源 Python/Make/YAML 或 installer 文件，不要求
`Makefile.ai`，不修改全局 Agent/MCP 配置，也不实现可选的宿主面板、controls 脚手架或
close-gap 便利功能；不改变固定的参考源或目标提交。

## 验收与证据

1. 九个 pinned 路径均已阅读，每个路径恰有一条非空且有证据支持的 inventory 记录。
2. 生成的 inventory 为本 WI 登记八条 `implemented-different-by-design` 和一条
   `reference-only`，本批没有 deferred 或 migrate gap。
3. 英文、简体中文、日文 comparison/parity 页面与本 Work Item 的 source pin、分类和边界一致。
4. 内部进度声明、源专属 fixture 以及未运行的 provider/enterprise assurance 不得作为当前 Runtime 事实。
5. 安装 Runtime 的 inspect/status/doctor、文档与 conformance 检查、生命周期收尾、hosted CI
   和精确清理提供终态证据；历史 evidence 不重写。

[English](WI-326-reference-file-comparison-batch-06.md) ·
[日本語](WI-326-reference-file-comparison-batch-06.ja.md)
