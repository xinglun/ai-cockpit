---
author: AI Cockpit maintainers
title: "WI-249——Parity finalization 登记"
workItemId: WI-249-parity-finalization-registration
description: "要求修改 parity 的 Work Item 在 verification 前登记 lifecycle-bound 终态路径。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-249-parity-finalization-registration
authority: canonical
---

# WI-249——Parity finalization 登记

WI-249 恢复不可变 WI-247 predecessor，并消除必须在 archive 后修改文档的顺序循环。它自己的
parity 行在 verification 前提交，预先列出未来 archived Contract、verification、canonical
finalize 与 close 路径。状态明确为条件式：`进行中 → 验证关闭后已实现`。在 PR #199 完成
审查、合并、finalization、准确 cleanup 与 close 前，它不宣称完成。

## 条件控制与质量 profile

治理完整性门禁检查 active Contract 的 scope/acceptance 与 active Summary 的 changed paths。
只有这些声明明确拥有 `docs/reference/reference-parity*` 或 parity registration 时，才要求
三条准确 lifecycle-bound 行。静态 selector 在 light profile 执行；standard 与 strict 继承。
普通非 parity 代码 Work Item 保持 `active_non_parity`，不会因为运行更宽 profile 就被扩大到
文档 scope。

已归档代码使用的 pending registry 仍是独立临时桥接；其 repository/PR/head/base/record 绑定、
registry-only append 拓扑、regular-file containment 与 default-branch stale 行为保持不变。

## Fail-closed evidence

回归确定性证明缺失、partial、仅终态、foreign path 与仅 archive 后投影全部失败。有效行由 Git
blame 定位引入 commit，并由 Git history 证明该 commit 严格早于 verification evidence 加入。
同一行 bytes 随后可通过 active、awaiting-merge-close 与 closed 状态，而无需改写 archive evidence。
