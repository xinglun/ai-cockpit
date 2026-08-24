---
author: AI Cockpit maintainers
title: "WI-252——Manifest gate 顺序恢复"
workItemId: WI-252-manifest-gate-order-recovery
description: "恢复 WI-245 不可变失败交付，并令 repository gate IDs 全局有序且唯一。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-252-manifest-gate-order-recovery
authority: canonical
---

# WI-252——Manifest gate 顺序恢复

WI-252 是 Runtime 记录的 WI-245 不可变失败交付 successor。predecessor recovery
receipt 绑定 WI-245 的 Contract、Summary、Outcome、Events、archive、verification 与
finalization digests；这些历史 bytes 留在本交付之外，且不会被改写。

## 验收边界

- `tests/ci/repository_gate_manifest.json` 中 gate IDs 按全局字典序排列且唯一，
  因此 `docs_pending_parity_registry_regression` 位于
  `docs_work_item_status_consistency` 之前。
- duplicate 与 out-of-order fixture manifests 在 route selection 前 fail closed，
  并使用 hosted quality 的同一校验。
- 在 `origin/main@87bfd866` 重放 WI-245 仍适用的文档状态、inventory 与发布 truth
  修改；不会把缺少 predecessor archive 的条目虚假登记为当前 parity Work Item。
- 固定比较继续保留 720 条 deferred 与准确 4 条 capability/profile
  `migrate-gap`。provider truth 保持 identity-bound、可检测漂移，而非宣称 immutable。

## 验证与生命周期

回归先精确复现 PR #203 的 `gate IDs must be deterministic` 失败，再在排序 ID 并
添加负向 fixtures 后通过 manifest 与 quality-route suites。还必须通过完整 docs、
governance、format、clippy、workspace、installed Runtime 与 exact-head hosted checks。
此 pre-archive 行引用未来 archived Contract、verification evidence、canonical
finalization 与 structured close；reviewed close 前不声明完成。

## 参考

- [WI-245 失败 predecessor](WI-245-doc-status-parity-recovery.zh-CN.md)
- [参考源 parity](../reference/reference-parity.zh-CN.md)
- [参考文件比较](../reference/reference-file-comparison.zh-CN.md)
- [发布与分发](../release/distribution.zh-CN.md)
