---
author: AI Cockpit maintainers
title: "WI-248——recovery decision 读侧恢复"
workItemId: WI-248-recovery-read-side-recovery
description: "从当前默认分支重新交付严格的 current recovery-decision 读侧校验，不导入失败 predecessor lifecycle。"
audience:
  - adopter
  - maintainer
status: current
lastVerifiedBy: WI-248-recovery-read-side-recovery
authority: canonical
---

# WI-248——recovery decision 读侧恢复

WI-242 在旧 base 上生成了 verified 不可变 archive 与 canonical pre-merge
finalization，但 hosted quality 因三语 parity 未登记而拒绝该交付；draft PR 继续保留失败
交付事实。WI-248 记录 successor decision，从 `origin/main@7d1bd78` 启动，只重放
`a3846e5` 中已审阅的 Rust 实现与回归测试；不导入 WI-242 archive、evidence、
finalization 或旧 lifecycle commit。

## Current 读侧边界

- 记录时仍在写入 append-only receipt 前校验 repository、Runtime、predecessor artifact、
  decision、时间戳与 successor identity。
- Outcome 与 archive consumer 会对每个 current recovery candidate 重复上述检查，并校验
  regular-file 与 digest-bound 文件名边界。
- foreign、stale、tampered、malformed 或 ambiguous candidate 会通过稳定的
  `recovery_decision_invalid:<code>` 诊断失败关闭；它不能移动 active artifacts，也不能把
  Outcome 变绿。
- 有效 successor 或 supersede decision 必须让现有 successor Contract 反向绑定同一
  repository、predecessor identity 与 predecessor Contract digest。

## 历史边界

已归档历史记录继续按 historical projection 可读。current-read validator 不会改写不可变
bytes，也不会把 legacy Runtime identity 变成新的 current failure。WI-242 继续保留在
PR #192，作为失败 predecessor；`.ai/decisions/WI-242-recovery-read-side.recovery.json`
把交接绑定到 WI-248。

## 验证

TDD 回归覆盖有效 current recovery、伪造 repository/Runtime 与 predecessor 绑定、记录后
predecessor/successor 篡改、非法 candidate 文件名、拒绝时 active artifacts 不变，以及历史
archive 兼容。documentation、parity、governance、formatting、Clippy、focused repository
suites 与 locked workspace suite 均为必需项。
