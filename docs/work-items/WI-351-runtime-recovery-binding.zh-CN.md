---
author: Ray
title: "WI-351——Runtime retry recovery receipt 绑定"
workItemId: WI-351-runtime-recovery-binding
description: "在 Runtime 自身投影状态后保持 retry recovery receipt 有效，同时继续对不可信证据 fail-closed。"
audience:
  - maintainer
  - reviewer
status: recovered
authority: translation
canonical: docs/work-items/WI-351-runtime-recovery-binding.md
lastVerifiedBy: WI-351-runtime-recovery-binding
terminalArchive: .ai/work-items/archive/WI-351-runtime-recovery-binding.contract.json
terminalVerification: .ai/evidence/WI-351-runtime-recovery-binding.verification.json
capabilityClaims:
  - recovery_receipt_binding
---

# WI-351——Runtime retry recovery receipt 绑定

[English](WI-351-runtime-recovery-binding.md) · [日本語](WI-351-runtime-recovery-binding.ja.md)

## 意图与边界

本 Work Item 修复共享 Rust Runtime 的 retry recovery 生命周期。retry 之后，Runtime
可能更新当前 Summary、Outcome 和 Events 投影；这些由 Runtime 自身生成的字节不应使
同一份 retry receipt 被误判为 foreign 或 stale。同时，foreign、stale、malformed 和
错误命名的证据仍必须 fail-closed。

实现范围仅包括 recovery binding 逻辑及其回归测试。Sentinel 业务代码、Provider
发现、交易决策、gate、execution、position sizing、全局配置和历史 archive 均不在范围内。

## 验证

- 回归测试覆盖 `retry → verify → preflight → finish`，并模拟 retry 后 Runtime 自身对
  projection 的更新。
- 既有 recovery negative path 继续拒绝不合法证据。
- 本地 `cargo fmt --all -- --check`、locked workspace tests 和 clippy 均通过；hosted
  验证由 [PR #318](https://github.com/xinglun/ai-cockpit/pull/318) 承载。

当前 Work Item 是 recovered historical predecessor。不可变 archive 与 recovery decision
仍是 lifecycle evidence 的来源；delivery 由 WI-353 继续，且不改写前置 bytes。
