---
author: AI Cockpit 维护者
title: "WI-358——v0.2.35 发布与生命周期入口兼容性"
workItemId: WI-358-release-v0-2-35
description: "发布 adopter 清理顺序修复，并防止历史 close 记录阻塞新 Work Item。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-358-release-v0-2-35
terminalArchive: .ai/work-items/archive/WI-358-release-v0-2-35.contract.json
terminalVerification: .ai/evidence/WI-358-release-v0-2-35.verification.json
terminalFinalization: .ai/decisions/WI-358-release-v0-2-35.finalize.json
terminalDecision: .ai/decisions/WI-358-release-v0-2-35.close.json
capabilityClaims: [release_distribution, lifecycle_entry_compatibility]
---

# WI-358——v0.2.35 发布与生命周期入口兼容性

[English](WI-358-release-v0-2-35.md) · [日本語](WI-358-release-v0-2-35.ja.md)

## 目标

将已合并的 adopter 验收顺序修复发布为公开的 v0.2.35 Release。新归档的
Work Item 继续使用 fail-closed close 门；同时把没有新标记的旧归档作为历史
数据处理，避免它们永久阻塞新 Work Item。

## 范围

- 新 archive manifest 写入显式 `closeRequired` 标记。
- 新入口只对带该标记的当前归档强制 close；无标记的旧 bytes 保持历史语义，
  无效或当前归档的 close 仍然阻塞。
- 增加历史归档与当前归档的 repository 回归测试。
- 统一 Cargo 版本和三语发布/版本文档，同时保留 v0.2.34 发布失败事实。
- 仅通过审核过的 hosted release workflow 发布，并在发布后验收真实公开制品。

## 边界

不重写历史 Contract、close、evidence 或 archive bytes；不推断人工决定，也不
修改外部 Homebrew tap。发布后失败仍记录 `releasePublished: true` 与验收失败。

## 验收

1. 所有 workspace package 与 `Cargo.lock` 为 0.2.35，标签为 `v0.2.35`。
2. 新 archive manifest 含 `closeRequired: true`；带标记但没有有效身份绑定 close
   的归档继续被阻塞。
3. 没有标记的历史归档不阻塞新 Work Item，也不被提升为当前绿色 Outcome。
4. 发布前文档、release policy、版本一致性和 workspace verification 全部通过。
5. hosted Release 绑定 manifest、`SHA256SUMS`、SBOM、provenance 与 staged adopter
   检查；公开验收证明下载 binary 身份、生命周期、隔离、证据复用和临时目录清理。

## 验证

Runtime lifecycle evidence、hosted PR checks、release workflow、公开 binary digest
和 adopter acceptance receipt 是权威记录。终态 lifecycle：archive
`.ai/work-items/archive/WI-358-release-v0-2-35.contract.json`；verification
`.ai/evidence/WI-358-release-v0-2-35.verification.json`；finalization
`.ai/decisions/WI-358-release-v0-2-35.finalize.json`；close
`.ai/decisions/WI-358-release-v0-2-35.close.json`。
