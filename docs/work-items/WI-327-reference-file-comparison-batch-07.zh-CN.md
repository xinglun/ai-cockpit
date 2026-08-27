---
author: AI Cockpit maintainers
title: "WI-327——参考源文件对比批次 07"
workItemId: WI-327-reference-file-comparison-batch-07
description: "逐个比对固定参考源的九个采用方、校准与长周期文档路径，并登记有证据支持的 Rust 原生边界。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-327-reference-file-comparison-batch-07
terminalArchive: .ai/work-items/archive/WI-327-reference-file-comparison-batch-07.contract.json
terminalVerification: .ai/evidence/WI-327-reference-file-comparison-batch-07.verification.json
terminalFinalization: .ai/decisions/WI-327-reference-file-comparison-batch-07.finalize.json
terminalDecision: .ai/decisions/WI-327-reference-file-comparison-batch-07.close.json
---

# WI-327——参考源文件对比批次 07

## 意图与边界

逐个比对固定参考提交
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` 中的以下九个路径。为采用方保留
校准、证据和长周期治理语义，不复制源 Python、Make、fixture、scanner 或内部进度实现。

共享 Rust Runtime 仍在工程外部安装，所有 repository 请求都必须显式携带 `--repo`。
本批是文档与 conformance 台账工作，不增加 Runtime 行为，也不宣称 source wire 兼容。

## 逐文件对比

| pinned 参考路径 | 分类 | Rust/adopter 对应与边界 |
| --- | --- | --- |
| `docs/reference/adopter-long-cycle-validation.ja.md` | `implemented-different-by-design` | 发布二进制采用方/升级验收及日文生命周期、安全路线保留隔离安装、生命周期、回滚和清理证据；不复制源多技术栈 fixture 与 Make/Python 编排。 |
| `docs/reference/adopter-long-cycle-validation.md` | `implemented-different-by-design` | 发布二进制采用方/升级验收及生命周期、安全路线保留隔离安装、生命周期、回滚和清理证据；不复制源多技术栈 fixture 与 Make/Python 编排。 |
| `docs/reference/adoption-reality-report.md` | `implemented-different-by-design` | Runtime capability/profile/status projection 与不可变 adopter receipt 区分模板能力、采用方执行、provider evidence 和企业 assurance。 |
| `docs/reference/bandit-synchronization-security-audit.md` | `reference-only` | 源专属 Bandit 历史发现与 digest 不是目标证据。目标没有 Python/Bandit 产品表面，Rust 原生质量门和 threat model 边界单独维护。 |
| `docs/reference/calibration-inventory.md` | `implemented-different-by-design` | 仓库绑定的 profile proposal/confirm、capability/status projection 和显式 unknown 保留事实/证据边界，不复制源 Python inventory。 |
| `docs/reference/calibration-profiles.ja.md` | `implemented-different-by-design` | 日文校准指南与严格 JSON profile policy 保留累积的 Lite/Standard/Strict 控制、人工选择、单调升级和明确降级证据。 |
| `docs/reference/calibration-profiles.md` | `implemented-different-by-design` | 校准指南与严格 JSON profile policy 保留累积的 Lite/Standard/Strict 控制、人工选择、单调升级和明确降级证据。 |
| `docs/reference/calibration-profiles.zh-CN.md` | `implemented-different-by-design` | 中文校准指南与严格 JSON profile policy 保留累积的 Lite/Standard/Strict 控制、人工选择、单调升级和明确降级证据。 |
| `docs/reference/calibration-session-model.ja.md` | `implemented-different-by-design` | 显式 proposal、确认与仓库绑定事实替代源内部 Session 模型；不引入通用交互 Session 或 checklist 权威。 |

## 采用方反馈边界

Cursor 采用方反馈是外部验收输入，不是新的 source wire 合同。当前 Runtime v0.2.33 已提供
稳定的生命周期 stdout JSON、面向人的 `work-item outcome`、close-before-next 入口门禁以及
fail-closed 的 start/verification 绑定。Cursor 必须显式安装仓库本地 adapter，并重放持久化
handoff，因为 IDE 无法被强制在聊天中展开 stderr。诊断 remediation、close-gap 便利命令和
controls 自动脚手架仍是独立产品决策，本 WI 不静默宣称已实现。

## 非目标

本 Work Item 不增加 Runtime 命令，不复制源 Python/Make/YAML、fixture 或 Bandit 文件，不要求
`Makefile.ai`，不修改全局 Agent/MCP 配置，也不实现可选的宿主面板、controls 脚手架或
close-gap 便利功能；不改变固定的参考源或目标提交。

## 验收与证据

1. 九个 pinned 路径均已阅读，每个路径恰有一条非空且有证据支持的 inventory 记录。
2. 生成的 inventory 为本 WI 登记八条 `implemented-different-by-design` 和一条 `reference-only`，本批没有 deferred 或 migrate gap。
3. 英文、简体中文、日文 comparison/parity 页面与本 Work Item 的 source pin、分类和边界一致。
4. 源专属 fixture/scanner 数量、内部进度及未运行的 provider/enterprise assurance 不得作为当前 Runtime 事实。
5. 安装 Runtime 的 inspect/status/doctor/agent doctor、文档与 conformance 检查、生命周期收尾、hosted CI 和精确清理提供终态证据；历史 evidence 不重写。

[English](WI-327-reference-file-comparison-batch-07.md) · [日本語](WI-327-reference-file-comparison-batch-07.ja.md)
