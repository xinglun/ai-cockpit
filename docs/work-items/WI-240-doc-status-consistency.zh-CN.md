---
author: AI Cockpit maintainers
title: "WI-240——文档状态与参考真值一致性"
workItemId: WI-240-doc-status-consistency
description: "WI-240 的不可变 PR 在 hosted governance 失败后，由 WI-245 恢复其文档治理交付。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-245-doc-status-parity-recovery
authority: canonical
---

# WI-240——文档状态与参考真值一致性

WI-240 在较旧的默认分支基线上生成了已验证 archive 与规范 pre-merge
finalization，但 PR #194 未合并。后续 release、parity 与 close 记录推进 `main` 后，
hosted governance 暴露了不可变的失败交付边界。它的归档 Contract、Summary、Outcome、
events、verification 与 finalization bytes 仍是保留在 predecessor 分支上的历史真值；
本文档不会导入或改写这些 bytes。

Runtime 生成的 successor receipt
`.ai/decisions/WI-240-doc-status-consistency.recovery.json` 绑定这些准确的 predecessor
digest，并把仍适用的状态、inventory 与 release truth 交付委托给基于
`origin/main@87bfd866` 的 WI-245。

## 恢复边界

- PR #194 已作为 superseded 关闭，且从未合并。
- WI-245 只重放实现内容，不重放 WI-240 lifecycle records。
- 固定的公开参考源 commit 保持不变。
- 保留 intervening Work Items 的 provider、release、SBOM、parity 与 terminal-decision 真值。

## 参考

- [WI-245 successor](WI-245-doc-status-parity-recovery.zh-CN.md)
- [参考源逐文件比较](../reference/reference-file-comparison.zh-CN.md)
- [参考源 parity](../reference/reference-parity.zh-CN.md)
- [发布与分发](../release/distribution.zh-CN.md)
