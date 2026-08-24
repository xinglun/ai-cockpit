---
author: AI Cockpit maintainers
title: "WI-240——文档状态与参考真值一致性"
workItemId: WI-240-doc-status-consistency
description: "将 Work Item 状态、参考 inventory、parity 与发布声明绑定到当前仓库证据，不改写历史治理 bytes。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-240-doc-status-consistency
authority: canonical
---

# WI-240——文档状态与参考真值一致性

本 Work Item 在 v0.2.31 Runtime 与 `origin/main` 比较基线上刷新文档真值；
不重新解释或改写已归档 Contract、evidence、decision 或已发布 Release 记录。

## 验收边界

- reference inventory 绑定 target commit
  `1c988ce9b04c3dcd45843f6577ed321457eeca0e`，忽略仅存在于 checkout 的漂移，
  并准确保留四条 capability/profile `migrate-gap` 与 720 条
  `deferred-next-batch` 记录。
- 英文、简体中文与日文 Work Item 文档的 identity、投影 status、verifier 一致。
  terminal 投影必须有仓库绑定的归档 Contract 与 close/recovery evidence；有歧义的
  跨文档 verifier 语义保持 unknown，不做猜测。
- 历史 recovery 允许 evidence-bound 显示投影：`Recovered` 可显示为 `historical`
  或 `recovered`；`Implemented` 只有在正文明确说明 immutable recovery history 时
  才可显示为 `recovered`。
- provider 报告 `immutable: false`，所以发布文档将 v0.2.31 描述为绑定身份且可检测
  漂移。仓库持久化的 adopter 基线是 `aarch64-apple-darwin`；hosted Linux workflow
  artifacts 仍是短期外部 evidence。

## 证据

确定性的 inventory、documentation acceptance 与 Work Item status 回归由
`.ai/evidence/WI-240-doc-status-consistency.verification.json` 和归档 Work Item
manifest 绑定。四个未解决的文件级 gap 继续保留在机器可读 inventory 中，不会因文档
投影而被关闭。

## 参考

- [参考源逐文件比较](../reference/reference-file-comparison.zh-CN.md)
- [参考源对齐](../reference/reference-parity.zh-CN.md)
- [发布与分发](../release/distribution.zh-CN.md)
