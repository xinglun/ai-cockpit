---
author: AI Cockpit maintainers
title: "WI-284——参考 Contract 语义终态 recovery"
workItemId: WI-284-reference-contract-semantics-terminal
description: "在 verification 之前完成终态 parity 绑定的有界 Rust Contract 语义批次。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-284-reference-contract-semantics-terminal
authority: canonical
---

# WI-284——参考 Contract 语义终态 recovery

WI-284 是不可变 WI-283 的显式 successor。它保留 predecessor 的全部 bytes，
并从最新审阅的默认分支完成同一有界实现。终态 parity 行及预期 evidence、decision
路径在 verification 之前提交，确保 archive truth 与 hosted quality 使用同一快照。
