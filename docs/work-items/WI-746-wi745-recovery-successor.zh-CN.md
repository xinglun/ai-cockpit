---
author: AI Cockpit maintainers
title: "WI-746——WI-745 parity 顺序恢复 successor"
description: "以先提交 parity 登记、后记录新 verification 证据的顺序重新交付有界文档恢复。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-746-wi745-recovery-successor
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-746-wi745-recovery-successor
---

[English](WI-746-wi745-recovery-successor.md) · [日本語](WI-746-wi745-recovery-successor.ja.md)

# WI-746——WI-745 parity 顺序恢复 successor

## 目的

保留 WI-745 的不可变 predecessor，并以明确的“先登记 parity、后记录
verification”边界重新交付其有界文档治理恢复。本 Work Item 不改变
WI-743 或 WI-745 的历史证据。

## 边界

本 Work Item 仅负责三语 WI-746 文档投影及其三条 reference-parity 台账
记录。不修改生产代码、Runtime 行为、治理规则、WI-743 或 WI-745 记录、
PR #718，或其他 agent 的 Work Item。

## 证据与生命周期

- WI-745 recovery decision 是明确的 predecessor 绑定。
- 先提交文档页面和 parity 行。
- 仅在该提交之后记录新的 Runtime verification 证据。
- 生命周期为 `start → preflight → checkpoint → verify → finish → archive
  → close`；在 finalization 前，终态路径保持 planned。

## 验收

文档验收、parity 状态检查和 Work Item 状态一致性检查必须通过。Git 历史
必须能够审查“先行登记、后有证据”的顺序，并且不得重写历史记录或生产
行为。
