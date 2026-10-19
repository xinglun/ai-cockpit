---
author: AI Cockpit maintainers
title: "WI-1090 — 分段当前材料审阅替换"
description: "分段当前材料审阅替换"
audience:
  - maintainer
  - reviewer
authority: explicit-user-authorization
workItemId: WI-1090-authority-source-material-review-replacement
status: in_progress
lastVerifiedBy: WI-1090-authority-source-material-review-replacement
contractDigest: sha256:c9a3bda349078f1fc15891e9e789d100952cbadaea12eb464f2024c90617ce9c
---

# WI-1090-authority-source-material-review-replacement — 分段当前材料审阅替换

human:Ray 对当前 Contract 身份的真实确认已由 agent:Codex 于2026-10-19T19:18:03Z原生记录，随后建立checkpoint1。实现与测试进行中，尚未声明产品验收通过。

H核验不可变旧收据及R→A已审原生历史，仅允许四类历史字段且profile全段不变；P证明最后相邻A→B只改变authoritySource；C独立绑定当前精确决定。旧收据仍invalid，不消除当前Unknown。

投影与持锁record共用有限证明；替换采用不可变sealed prepare与sidecar、一次Summary原子提交及精确pending恢复。evidenceRefs格式不认证真人，self_declared保证不升级。

范围仅两份Rust文件与六份治理文档。当前候选七项正式检查尚未执行。不操作Sentinel、真实消费者修约、真实工作项终态、合并、发版或Runtime采用。

Active [Contract](../../.ai/work-items/active/WI-1090-authority-source-material-review-replacement.contract.json): `sha256:c9a3bda349078f1fc15891e9e789d100952cbadaea12eb464f2024c90617ce9c`. Runtime evidence remains authoritative.

Spec: `../../.ai/evidence/WI-1090-authority-source-material-review-replacement/segmented-v4-review/segmented-authority-source-replacement.v4.proposal-only.json`.
