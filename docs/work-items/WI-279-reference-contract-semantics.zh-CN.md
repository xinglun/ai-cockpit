---
author: AI Cockpit maintainers
title: "WI-279——参考 Contract 语义 predecessor"
workItemId: WI-279-reference-contract-semantics
description: "首个严格 Contract parity 批次的不可变 predecessor 记录；实现由 WI-280 继续。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-280-reference-contract-semantics-successor
terminalArchive: .ai/work-items/archive/WI-279-reference-contract-semantics.archive.json
terminalVerification: .ai/evidence/WI-279-reference-contract-semantics.verification.json
terminalDecision: .ai/decisions/WI-279-reference-contract-semantics.recovery.24e2fb7f991584a201968d617a09602fdaef6a8fd87d1bc59e052848fa18bde3.json
authority: canonical
---

# WI-279——参考 Contract 语义 predecessor

WI-279 在 reviewed branch snapshot 完成最终化之前到达不可变的
`finish_ready` 边界。Runtime 记录 retry、successor 和 supersession 决定，
不改写 predecessor evidence；WI-280 在新 snapshot 上继续同一有界实现。
