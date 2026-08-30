---
author: AI Cockpit maintainers
title: "WI-426——恢复 successor 绑定兼容"
description: 保持 successor 严格 lineage，同时安全识别已完成的历史 successor。
workItemId: WI-426-recovery-successor-binding
audience: [contributor, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-426-recovery-successor-binding
---

# WI-426——恢复 successor 绑定兼容

## 意图

修复不可变 archived predecessor 已有合法 successor recovery receipt、但旧版
successor Contract 缺少新版 predecessor 字段的生命周期缺口。新 Runtime 创建的
successor 仍必须严格绑定。

## 范围

- 新 successor Contract 必须绑定 predecessor Work Item、Contract digest、recovery path
  和 repository identity。
- 只有终态 evidence 完整的历史 successor 才能进入明确的兼容路径，并在新的
  append-only recovery receipt 中标记。
- foreign、stale、malformed、symlink、不完整或被篡改的记录必须拒绝。
- 保留 predecessor bytes，并同步三语边界说明。

## Evidence 边界

历史兼容不是新的 green 结果。它必须具备有效 archive manifest、严格 verification
evidence 和已确认的结构化 close。兼容标记为
`successorBindingMode: legacy_terminal_evidence`。

[English](WI-426-recovery-successor-binding.md) · [日本語](WI-426-recovery-successor-binding.ja.md)
