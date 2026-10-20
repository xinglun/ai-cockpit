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
contractDigest: sha256:1cc7ec218c2033ed48feb6b1ac298b392485dc898dcefac58f0fee14d06895c4
---

# WI-1090-authority-source-material-review-replacement — 分段当前材料审阅替换

真人 Ray 已确认审阅过的原生计划，checkpoint1 后完成实现。前项的七项声明原生检查通过，并已完成自身原生 finish 与 archive。[PR #1050](https://github.com/xinglun/ai-cockpit/pull/1050) 仍为 Draft，HEAD 为 `cba3ea1661749ebbf1929bc523c5b86ff10eb7e7`；CI `39344530319` 在 Windows checkout 和文档门禁失败。原生验证不能替代托管 CI 验收。close 仍待处理。

H 校验不可变旧收据、已审阅 R→A 原生历史、四个允许的历史路径及不变 profile。P 证明相邻 A→B 只改变 authoritySource。C 绑定独立且精确的当前决定。旧收据仍无效，不能解除当前 Unknown。

替换使用共享 projection/锁内证明、不可变 sealed prepare 与 sidecar，并通过一次原子 Summary 提交及精确 pending recovery 完成。evidenceRefs 语法不能认证真人；self_declared 保证级别不变。原生等效合成 fixture 保留实际 20-operation 混合历史形状，并在相邻来源绑定前独立改变当前材料。未证明物理多文件原子性。

选定文档后继 `WI-1090-evidence-docs-recovery` 不改变原两项 Rust 文件和历史原生记录；它使用显式 locator 修复八份 Agent 副本的存储路径，并修正三语派生投影。选择记录是 `.ai/decisions/WI-1090-authority-source-material-review-replacement.recovery.json`。这里不证明任一工作项 close、合并、发布、Runtime 采用、真实消费者验收或 Sentinel 恢复。

归档 [Contract](../../.ai/work-items/archive/WI-1090-authority-source-material-review-replacement.contract.json)：`sha256:1cc7ec218c2033ed48feb6b1ac298b392485dc898dcefac58f0fee14d06895c4`。原生验证：`.ai/evidence/WI-1090-authority-source-material-review-replacement.verification.json`。以 Runtime 实际证据为准。

M6 副本 locator：`.ai/evidence/WI-1090-authority-source-material-review-replacement/m6-short-20261020/path-map.actual.json`；原 manifest 逻辑键与字节不变。原始规格：`.ai/evidence/WI-1090-authority-source-material-review-replacement/segmented-v4-review/segmented-authority-source-replacement.v4.proposal-only.json`。
