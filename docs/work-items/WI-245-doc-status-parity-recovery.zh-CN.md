---
author: AI Cockpit maintainers
title: "WI-245——文档状态与 parity 恢复"
workItemId: WI-245-doc-status-parity-recovery
description: "在当前 main 恢复 WI-240，并将 stale conditional 文档绑定到仓库终态证据。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-252-manifest-gate-order-recovery
authority: canonical
---

# WI-245——文档状态与 parity 恢复

WI-245 曾是 Runtime 记录的 WI-240 不可变失败交付 successor；随后因 repository gate IDs
未按全局顺序排列而在 hosted quality 失败。
`.ai/decisions/WI-245-doc-status-parity-recovery.recovery.json` 保留该不可变失败，
并将当前重新交付交给 WI-252。

## 验收边界

- 确定性 reference inventory 从固定 Git trees 派生，不受 dirty/untracked checkout
  metadata 影响，并保留 720 条 deferred 与准确 4 条 capability/profile `migrate-gap`。
- 三语 Work Item 状态与规范 archived Contract 加 close/recovery evidence 对照；已关闭
  Work Item 若仍带 conditional 或 after-close parity 文案，必须确定性失败。
- WI-241、WI-249 与 WI-251 的终态行绑定各自 archived Contract、verification evidence、
  canonical finalization、sequence-2 deleted cleanup transition 与结构化 close decision。
- v0.2.31 因 provider truth 为 `immutable: false` 而保持 identity-bound、可检测漂移；
  持久化 adopter 基线是 `aarch64-apple-darwin`，hosted Linux run `32696048024`
  仍是 provider-retained 外部 evidence。

## 验证与生命周期

WI-245 的不可变 archive 不在 successor 交付内，因此本分支不会把 WI-245 登记为当前 parity
Work Item。WI-252 只重放仍适用的实现与文档修改，并必须完成自身 Runtime verification、
hosted review 与 structured close。

## 参考

- [WI-240 predecessor](WI-240-doc-status-consistency.zh-CN.md)
- [参考源逐文件比较](../reference/reference-file-comparison.zh-CN.md)
- [参考源 parity](../reference/reference-parity.zh-CN.md)
- [发布与分发](../release/distribution.zh-CN.md)
