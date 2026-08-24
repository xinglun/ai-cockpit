---
author: AI Cockpit maintainers
title: "WI-253——关闭后文档终态化"
workItemId: WI-253-docs-terminalization
description: "依据不可变 close 证据终态化 WI-252 文档，并拒绝新关闭 Work Item 的条件式状态。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-253-docs-terminalization
terminalArchive: .ai/work-items/archive/WI-253-docs-terminalization.contract.json
terminalVerification: .ai/evidence/WI-253-docs-terminalization.verification.json
terminalFinalization: .ai/decisions/WI-253-docs-terminalization.finalize.1ccec42e056dd7eac857ba49d1dc2becd6e2ba21f6461a62599e18101d986293.json
terminalDecision: .ai/decisions/WI-253-docs-terminalization.close.json
authority: canonical
---

# WI-253——关闭后文档终态化

WI-253 是已合法关闭 WI-252 的有界 Runtime successor。recovery decision 绑定
WI-252 的 canonical Contract、Summary、Outcome、Events digests，以及 archive、
verification、sequence-2 finalization 与 structured close 证据；不会编辑其中任何
不可变记录。

## 验收边界

- WI-252 的英文、简体中文、日文 Work Item 文档与 reference-parity 行使用终态
  `implemented` / `已实现` truth，并引用准确的持久化终态证据路径。
- status-consistency 回归拒绝新治理终态 Work Item 任一语言 counterpart 中的条件式
  lifecycle 文案。不会追溯改写 WI-252 enforcement boundary 之前的历史文档。
- reference inventory 的 target working-tree count 警告来自故意漂移的负向 fixture。
  canonical count 与 digest 仍与固定 commit 归一，因此无需更改生产 checker。

## 验证与 lifecycle

focused 回归先证明各语言条件式文案曾被接受，随后在真实 stale WI-252 projection
上失败，并仅在投影终态证据后通过。此 active 登记列出未来 WI-253 archive、
verification、finalization 与 close 路径；它本身不是终态证据。

## 参考

- [WI-252 predecessor](WI-252-manifest-gate-order-recovery.zh-CN.md)
- [Reference parity](../reference/reference-parity.zh-CN.md)

