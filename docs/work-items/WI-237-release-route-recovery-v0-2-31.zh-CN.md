---
author: AI Cockpit maintainers
title: "WI-237——发布路由恢复与 v0.2.31 发布"
workItemId: WI-237-release-route-recovery-v0-2-31
description: "修复清洁批次的发布路由，并在不改写 v0.2.30 历史的前提下发布下一份不可变补丁版本。"
audience:
  - maintainer
  - reviewer
  - adopter
status: recovered
authority: canonical
lastVerifiedBy: WI-237-release-route-recovery-v0-2-31
---

# WI-237——发布路由恢复与 v0.2.31 发布

本 Work Item 修复清洁批次边界暴露的发布质量路由问题：没有 active Work Item
目录是合法状态，但发布 workflow 发现 Contract 时不能因此失败。不可变的
v0.2.30 标签及其失败发布尝试作为历史事实保留，不会被改写。修复后的路由发布
v0.2.31，公开 adopter 与 N-1 验收交由 successor Work Item 负责。

## 验收边界

- 发布路由在 `.ai/work-items/active` 不存在时仍确定性通过，并有零 active
  Work Item 回归测试。
- package metadata、lockfile、发布文档和三语 parity 标识 v0.2.31；v0.2.30
  作为失败的不可变历史保留。
- Hosted release checks 只发布不可变的 v0.2.31 artifact。
- v0.2.31 的公开 artifact identity、安装 Runtime 检查以及隔离 adopter/upgrade
  验收交给 successor Work Item。

## 参考

- [发布与分发](../release/distribution.zh-CN.md)
- [版本策略](../architecture/versioning.zh-CN.md)
- [参考 parity ledger](../reference/reference-parity.zh-CN.md)
