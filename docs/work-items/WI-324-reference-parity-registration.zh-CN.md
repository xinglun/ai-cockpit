---
author: AI Cockpit maintainers
title: "WI-324——参考 parity 登记恢复"
workItemId: WI-324-reference-parity-registration
description: "修复不可变 WI-323 归档后由托管文档治理门发现的登记遗漏。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-324-reference-parity-registration
terminalArchive: .ai/work-items/archive/WI-324-reference-parity-registration.contract.json
terminalVerification: .ai/evidence/WI-324-reference-parity-registration.verification.json
terminalFinalization: .ai/decisions/WI-324-reference-parity-registration.finalize.json
terminalDecision: .ai/decisions/WI-324-reference-parity-registration.close.json
---

# WI-324——参考 parity 登记恢复

## 意图和目标

修复托管 `docs_governance_integrity` 门在 WI-323 不可变归档后发现的三语
`reference-parity` 登记遗漏。保留 WI-323 的不可变归档和失败交付历史，让恢复的
successor 可审计、可独立评审。

## 范围和边界

在英文、简体中文、日文 parity ledger 中登记 WI-323（不可变的已恢复前置）和
WI-324（有界 successor）。携带已评审的 WI-323 inventory、comparison、Human Benefit
页面、conformance generator/test 及三语 Work Item 记录；新增三语 WI-324 记录，并在
创建新 PR 前运行同一套文档/conformance 检查。

不改写前置 archive、evidence 或 recovery bytes；不新增 Runtime 功能、不改变 CI
policy、不复制源 Python/Make 文件，也不修改全局 Agent/MCP 配置。

## 验收和验证

1. 三个 parity ledger 均登记 WI-323 和 WI-324，链接、状态和恢复说明一致。
2. 从干净 `origin/main` 基线运行携带的 inventory 和文档测试并通过。
3. 托管 `docs_governance_integrity` 及其他必需 PR 检查全部通过。
4. 前置 archive digest 和 recovery binding 保持不变，successor Contract/evidence
   绑定显式 repository context。

## 恢复证据

前置 archive 和托管失败由
`.ai/decisions/WI-323-reference-documentation-foundation.recovery.json` 引用。
只有在前置已归档后才发现遗漏，因此建立本 successor；不引入新功能范围。

[English](WI-324-reference-parity-registration.md) ·
[日本語](WI-324-reference-parity-registration.ja.md)
