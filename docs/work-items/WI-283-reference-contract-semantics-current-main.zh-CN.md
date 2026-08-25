---
author: AI Cockpit maintainers
title: "WI-283——当前 main 的参考 Contract 语义"
workItemId: WI-283-reference-contract-semantics-current-main
description: "在最新审阅的默认分支上重新验收有界 Rust Contract 语义 parity 批次，并保留 WI-282 的不可变历史。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-283-reference-contract-semantics-current-main
authority: canonical
---

# WI-283——当前 main 的参考 Contract 语义

WI-283 是不可变 WI-282 的显式 successor。它从默认分支修订
`622836157e945a46f8cb34ee747084d3193e7f28` 重新验收同一有界 Contract 语义
实现，同时保留 predecessor 的 Contract、evidence、archive 和 recovery bytes。
predecessor 不被改写；其 hosted quality 拒绝作为 recovery 历史保留。
