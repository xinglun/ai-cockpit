---
author: AI Cockpit maintainers
title: "WI-401——v0.2.40 公开 Release 发布与 adopter 验收"
description: "发布经过审查的 v0.2.40 Runtime，并在全新 adopter 中验收不可变制品。"
workItemId: WI-401-release-v0-2-40-publication
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-401-release-v0-2-40-publication
capabilityClaims: [release_distribution, adopter_acceptance, runtime_installation]
---

# WI-401——v0.2.40 公开 Release 发布与 adopter 验收

[English](WI-401-release-v0-2-40-publication.md) · [日本語](WI-401-release-v0-2-40-publication.ja.md)

## 意图

从经过审查且同步的 `main` 发布 v0.2.40，并证明不可变公开制品可以治理全新的
adopter 以及当前工程。

## 边界

本 Work Item 只处理标签/Release 发布、不可变制品验收、已验证 binary 的安装和可审计
外部验收证据。不修改 Runtime 语义、参考源对齐、对象工程代码或全局 Agent/MCP 配置。
公开验收必须拒绝源码构建和 workspace binary 回退。

## 验收

1. v0.2.40 标签和公开 Release 从经过审查的 main 合并产出，并通过 Release identity、
   SBOM、provenance 和 checksum 门禁。
2. 下载制品记录 tag、version、archive digest、binary digest、platform 和 download
   source，并将这些 identity 绑定到证据。
3. 全新 adopter 拥有独立 repository identity 和完整生命周期；`first-adopter-smoke`
   保持 `not_ready`，证明 evidence reuse，且 forbidden roots 与临时 run root 均已清理。
4. 已验证公开 binary 安装到当前工程后报告 `COMPATIBLE` 和 `doctor=ok`；main 最终同步并
   ready on base。

## 验证边界

发布后验收只是公开制品的证据，不能改写 Release 真相或历史证据。验收失败必须记录
`releasePublished: true` 与 `adopterAcceptance: failed`。
