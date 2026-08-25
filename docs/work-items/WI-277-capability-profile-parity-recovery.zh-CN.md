---
author: AI Cockpit maintainers
title: "WI-277 — capability profile parity recovery"
workItemId: WI-277-capability-profile-parity-recovery
description: "在 WI-276 恢复后补回托管 parity 注册，并证明 adopter 工程可以继承 capability/profile。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-277-capability-profile-parity-recovery
terminalArchive: .ai/work-items/archive/WI-277-capability-profile-parity-recovery.contract.json
terminalVerification: .ai/evidence/WI-277-capability-profile-parity-recovery.verification.json
terminalFinalization: .ai/decisions/WI-277-capability-profile-parity-recovery.finalize.26a5046378afcc467b75e703bb6b7dd83d53f665d76605695f7f28a6b9b8f564.json
terminalDecision: .ai/decisions/WI-277-capability-profile-parity-recovery.close.json
authority: canonical
---

# WI-277 — capability profile parity recovery

## Intent

补回前一个 Work Item 遗漏的三语 reference-parity 注册，并确认 repository-bound
CLI 与 MCP 投影能够提供严格的 capability/profile 声明。

## Scope

- 保留 WI-276 不可变的 recovery linkage。
- 在 verification 之前注册英语、日语、中文三份 parity 行。
- 验证两个 repository 的隔离、malformed/stale 声明拒绝和只读 adopter 投影。
- 绑定 reviewed PR、合并观察、精确 cleanup 与 close decision。

## Boundary

不改写 WI-276 的 archive/evidence bytes，不新增 capability 语义，不修改全局
Agent/MCP 配置，也不执行后续 architecture cleanup。

## Acceptance and verification

- 在一次有边界的 Runtime 执行中通过 Rust、文档、conformance 与 governance gates。
- 通过 hosted quality、Windows Runtime 和 V1 behavioral oracle。
- 用 Runtime receipt 记录合并以及 branch/worktree cleanup。

