---
workItemId: WI-390-reference-style-guide
title: “参考 Work Item 编写指南”
author: AI Cockpit 维护者
description: “固定 Work Item 编写指南的语义比较记录。”
audience:
  - maintainer
  - reviewer
authority: canonical
status: implemented
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
lastVerifiedBy: WI-390-reference-style-guide
terminalArchive: .ai/work-items/archive/WI-390-reference-style-guide.contract.json
terminalVerification: .ai/evidence/WI-390-reference-style-guide.verification.json
terminalFinalization: .ai/decisions/WI-390-reference-style-guide.finalize.b0a9c123b5f157c327a4068001f478d05b6d39e152363bc167945e0dc83fe423.json
terminalDecision: .ai/decisions/WI-390-reference-style-guide.close.json
---

# WI-390——参考 Work Item 编写指南

[English](WI-390-reference-style-guide.md) · [日本語](WI-390-reference-style-guide.ja.md)

## Intent

逐段比较固定的 `docs/work-item-style-guide.md`，只把其中面向读者的治理语义带入 Rust-native
文档。安装实现和 Runtime 实现明确不复制。

## Scope

- 固定源文件：`docs/work-item-style-guide.md`
- Rust 对应：`docs/reference/work-item-style-guide.*`
- 为本次比较同步 index、parity 和 inventory

## Acceptance

- 表达先说明结果、明确问题/边界/非目标、可观察验收、由人拥有的决定、可执行验证、相称流程以及先文档后 schema。
- 说明共享 Runtime 和显式 `--repo` 的仓库隔离；不复制安装命令或源 Runtime 代码。
- 三语链接和比较记录保持一致。

## Verification boundary

这是语义/文档对等，不是源命令、JSON wire 或 provider 状态兼容。对象/采用方工程通过自己的 `.ai/` 和
adapter 继承面向读者的规则，而 Contract、evidence、knowledge 和 repository identity 仍按仓库隔离。

## Evidence

Runtime 将在以下位置记录终态证据：

- `.ai/evidence/WI-390-reference-style-guide.verification.json`
- `.ai/work-items/archive/WI-390-reference-style-guide.*`
- `.ai/decisions/WI-390-reference-style-guide.close.json`
