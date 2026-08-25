---
author: AI Cockpit maintainers
title: "WI-280——参考 Contract 语义 successor"
workItemId: WI-280-reference-contract-semantics-successor
description: "对参考 parity 字段实施严格 Rust Contract 校验与 fail-closed preflight review。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-280-reference-contract-semantics-successor
terminalArchive: .ai/work-items/archive/WI-280-reference-contract-semantics-successor.archive.json
terminalVerification: .ai/evidence/WI-280-reference-contract-semantics-successor.verification.json
terminalFinalization: .ai/decisions/WI-280-reference-contract-semantics-successor.finalize.json
terminalDecision: pending-reviewed-merge-close
authority: canonical
---

# WI-280——参考 Contract 语义 successor

WI-280 在新 snapshot 上继续不可变的 WI-279 predecessor。它严格校验
scenario coverage、acceptance criteria 和 concurrency boundary，使 malformed
声明 fail-closed，并同步三种语言的 Rust 映射文档。
