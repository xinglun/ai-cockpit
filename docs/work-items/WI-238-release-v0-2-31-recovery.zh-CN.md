---
author: AI Cockpit maintainers
title: "WI-238——v0.2.31 发布恢复"
workItemId: WI-238-release-v0-2-31-recovery
description: "在不可变的 WI-237 恢复历史之后，从干净默认分支重新交付 v0.2.31。"
audience:
  - maintainer
  - reviewer
  - adopter
status: implemented
authority: canonical
lastVerifiedBy: WI-238-release-v0-2-31-recovery
---

# WI-238——v0.2.31 发布恢复

WI-238 是 WI-237 的清洁 successor。WI-237 的不可变归档和 pre-merge
finalization receipt 被保留，因为 hosted quality 暴露了三语 parity 绑定缺失，
而尝试中的修复又推进了未合并分支的 head。本 Work Item 从同步后的默认分支重新
交付同一有界发布修复。

## 验收边界

- 发布质量路由在没有 active Work Item 目录时仍确定性通过，并有回归测试。
- 三语 parity 行在 hosted checks 运行前绑定 verification evidence 与 pre-merge
  finalization receipt。
- 不改写或复用失败的不可变 v0.2.30 标签；只有 reviewed merged head 的 hosted
  checks 通过后才发布 v0.2.31。
- 公开 v0.2.31 和 N-1 upgrade 只使用下载的不可变 artifact，并隔离根目录、清理
  临时运行根目录。

## 恢复边界

WI-237 作为不可变历史 recovery evidence 保留。successor 通过
`.ai/decisions/WI-237-release-route-recovery-v0-2-31.recovery.json` 绑定。
不改写 predecessor 的 Contract、Summary、Outcome、Events、verification、archive
或 finalization receipt。

## 参考

- [参考 parity ledger](../reference/reference-parity.zh-CN.md)
- [WI-237 不可变 Work Item](WI-237-release-route-recovery-v0-2-31.zh-CN.md)
- [发布与分发](../release/distribution.zh-CN.md)
