---
author: AI Cockpit maintainers
title: "WI-313——close 后 finalization reconciliation"
workItemId: WI-313-post-close-finalization-reconciliation
description: "强制先清理再 close，并为不可变的历史 close 记录提供严格绑定的恢复路径。"
audience:
  - maintainer
  - reviewer
status: recovered
authority: canonical
lastVerifiedBy: WI-313-post-close-finalization-reconciliation
---

# WI-313——close 后 finalization reconciliation（已恢复的历史记录）

## 意图与边界

W312 暴露了真实的顺序缺口：旧 Runtime 可能在 provider finalization 仍为
`retained` 时写入 close，随后 closed-document promotion gate 正确拒绝宣称清理完成。
WI-313 曾尝试交付有界修正，但 PR #277 在合并前被 hosted quality 正确拒绝。
因此本文描述的是不可变的失败交付历史，不是已合并实现。WI-321 记录明确的
successor-owned recovery；WI-313 的任何字节都不改写。新的 Work Item 必须在 close 前
清理 provider 资源；只有不可变的历史 close 才允许之后追加一条绑定的 deleted transition。

## 范围与验收

原始 Rust protocol/repository lifecycle 修正及其 hosted 交付尝试作为历史 evidence 保持
不变。当前 gate 只有在 Runtime 生成且绑定 WI-321 的 successor receipt 存在时，才把本
Work Item 投影为 `已恢复`。原始 Contract、Summary、Outcome、Events、archive、verification、
retry receipt、branch 和 PR 字节都保持不变。文档 promotion gate 与三语 workflow 拒绝
孤立 retry，并要求明确 successor 或有效终态路径。

## 验证

原始 Rust finalization 定向测试与 hosted PR evidence 属于历史记录。WI-321 增加孤立 retry
静态回归，并验证三语恢复投影、文档门禁及由已安装 Runtime 生成的 successor receipt。发布
验收不以源码构建作为替代。

[WI-321 successor recovery](WI-321-explicit-failed-delivery.zh-CN.md)

[English](WI-313-post-close-finalization-reconciliation.md) ·
[日本語](WI-313-post-close-finalization-reconciliation.ja.md)
