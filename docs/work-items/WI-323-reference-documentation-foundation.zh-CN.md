---
author: AI Cockpit maintainers
title: "WI-323——参考文档基础"
workItemId: WI-323-reference-documentation-foundation
description: "逐个比对固定参考源的九个文档路径，记录 Rust-native adopter 与 Agent 边界。"
audience:
  - maintainer
  - reviewer
status: recovered
authority: canonical
lastVerifiedBy: WI-323-reference-documentation-foundation
---

# WI-323——参考文档基础

## 意图和目标

在固定参考源提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 上逐个比对接下来的
九个 deferred 路径。为本仓库和未来对象工程保留有用的治理语义，同时保持共享
Runtime 在工程外部、各 repository 状态隔离，并让所有命令显式带 `--repo`。

用户提供的 Cursor adopter 反馈作为本批次外部观察输入。先按当前 Runtime 实测和代码
确认它，而不是假设：生命周期 stdout JSON、可见人类 handoff/重放、repository
entry gate 和 start 前清洁检查已经实现。Cursor 聊天面板展示、诊断修复、close-gap
便利命令和可选 controls 脚手架是独立的产品决定。

## 比对文件

- `docs/contributing/installation-document-maintenance.md`
- `docs/current/README.md`
- `docs/design/harden-work-item-pr-closure.md`
- `docs/distribution.md`
- `docs/enterprise-security-boundary.md`
- `docs/examples/trust-layer-demo.sh`
- `docs/features/human-benefit-report.md`
- `docs/features/human-benefit-report.zh-CN.md`
- `docs/features/human-benefit-report.ja.md`

每个路径都有台账分类和非空理由。八条是 Rust-native 的
`implemented-different-by-design`，离线 trust demo 是 `reference-only`，本批
不隐藏任何 `migrate-gap`。

## 范围和边界

本批更新比对 inventory/generator 与回归断言、三语 reference comparison 页面、三语
Human Benefit Report 页面以及本三语 Work Item 记录；明确记录源 Make/Python/installer/
demo 边界、语义而非 wire/字节 parity，以及一份共享 Runtime、私有 repository-local
`.ai/` 状态的对象工程模型。

本批不新增 Runtime 命令、不改变 lifecycle 语义、不复制源 Python/Make/YAML/JSON
wire 文件、不要求 `Makefile.ai`、不改全局 Agent/MCP 配置、不重写历史 evidence，
也不发布 Release。

## 验收和验证

1. 九个固定源文件均已读取并逐项给出有证据的对应物或明确 reference-only 决定。
2. 生成 inventory 恰有九条 WI-323 记录：八条
   `implemented-different-by-design` 和一条 `reference-only`；本批没有 deferred
   或 migrate-gap。
3. 英文、简体中文、日文的 comparison 与 Human Benefit Report 页面语义一致，并互相链接。
4. 面向人的输出说明 `work-item outcome --repo ...`、MCP `work_item_outcome`、
   stdout 生命周期 JSON 与 human handoff 的区别、报告顺序、evidence 计数语义、过期/
   损坏时停止，以及 Contract authored acceptance 原文保留规则。
5. 记录不宣称 CLI 可以展开 Cursor 聊天面板，也不宣称目标提供源专有的
   `implementation_approach_report`、Make/Python generator 或 trust-demo authority。
6. 安装的 Runtime 以当前 repository context 验证，声明的检查全部通过，且没有修改无关
   repository bytes。

## 证据

不可变的源与目标 baseline 记录在 active Contract。生成 inventory、文档 acceptance
输出、diff 检查和 Runtime verification receipt 是本批次的权威证据。

[English](WI-323-reference-documentation-foundation.md) ·
[日本語](WI-323-reference-documentation-foundation.ja.md)
