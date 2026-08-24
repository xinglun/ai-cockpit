---
author: AI Cockpit maintainers
title: "WI-244——Pending parity registry"
workItemId: WI-244-pending-parity-registry
description: "为必须由独立文档变更交付的 parity 行增加严格 typed、fail-closed 的合并前登记。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-246-pending-parity-merge-ref-recovery
authority: canonical
---

# WI-244——Pending parity registry

代码 Work Item 可以合法完成 archive 与合并前 finalization，却没有权限修改三语 parity
账本。强制同一 PR 添加这些行会造成 scope 与 finalization head 死锁。WI-244 增加严格的
pending registry，且不复制或改写 predecessor `.ai` bytes。

## 边界

- registry 默认为空，不是通用豁免列表。
- pending 条目绑定 repository、完整 Work Item、provider PR、Contract base、canonical
  finalization head、registry 追加父提交、准确 record 路径、三条准确“进行中”行及创建时间。
- 正常 archive、verification 与 finalization 检查仍具有优先权。
- 只允许延后缺失的三语 parity 行；foreign、malformed、missing、mismatched、symlink、
  duplicate、stale、merged、partial 或 unrelated 输入全部 fail closed。
- 合并后的文档变更必须原子加入全部三语行并删除 pending 条目，不能修改 predecessor 历史。

## 验证

聚焦回归覆盖合法 Git 拓扑，以及 foreign、head/base/PR/path/row mismatch、duplicate-key、
missing record、symlink、unrelated append、partial row 与 default branch。Manifest 与 route
测试要求 light、standard、strict 三种 profile 都执行该回归。

## 恢复

WI-244 已在 PR #196 达到 verified archive 与合并前 finalization。随后 hosted merge ref
把 feature tree 与默认分支上的权威 WI-243 close receipt 合并，暴露了不可变 predecessor
snapshot 之外的三语 parity 漂移。Runtime recovery receipt
`.ai/decisions/WI-244-pending-parity-registry.recovery.json` 将准确的 Contract、Summary、
Outcome 与 Events digest 绑定到 WI-246；predecessor archive、evidence、finalization、PR
及 hosted-run bytes 均保持不变。
