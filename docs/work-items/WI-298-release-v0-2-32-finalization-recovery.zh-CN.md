---
author: AI Cockpit maintainers
title: "WI-298——v0.2.32 发布收尾恢复"
workItemId: WI-298-release-v0-2-32-finalization-recovery
description: "在不改写不可变归档的前提下，补齐 WI-297 缺失的 reviewed resource-finalization 链。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-298-release-v0-2-32-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-298-release-v0-2-32-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-298-release-v0-2-32-finalization-recovery.verification.json
terminalFinalization: .ai/decisions/WI-298-release-v0-2-32-finalization-recovery.finalize.json
terminalDecision: .ai/decisions/WI-298-release-v0-2-32-finalization-recovery.close.json
authority: canonical
---

# WI-298——v0.2.32 发布收尾恢复

## 意图

恢复 WI-297 归档后发现的缺失 `finalize-plan` 边界。前置 Work Item 的归档、
verification、recovery receipts 和已合并 PR 保持不可变；本 Work Item 只记录
有界的关闭恢复。

## 范围

- 绑定准确的 PR #258、分支、工作树和默认分支上下文。
- 用已安装 Runtime 为本恢复记录执行 verification 和 hosted quality checks。
- 记录 provider finalization，验证精确清理，并生成结构化 human close receipt。
- 保持前置/后继关系与全部证据的 identity binding。

## 范围外

发布实现、Runtime 行为、包元数据、adopter 验收、Homebrew 发布以及历史归档
改写均不在本恢复范围内。

## 验收

- WI-297 归档 bytes 保持不变，并由 recovery decision 引用。
- `finalize-plan` 在后继 verification 和 archive 之前记录。
- reviewed successor PR 的 hosted checks 通过。
- `finalize-verify` 在结构化 close 前证明准确的 feature 分支/工作树已清理。
- 可见的人类 Outcome 包含状态、未知项、证据、决定和下一步。

## Verification

使用显式 `--repo` 的已安装 Runtime、仓库治理和文档 gate、hosted quality checks，
以及完整的 `finalize-plan → finalize → finalize-verify → close` 链。

reviewed PR 的 hosted quality 结果属于终态证据；之前缺少 verification evidence
的 pre-archive 运行保持为历史失败，不会被重用。
