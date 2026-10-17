---
author: AI Cockpit maintainers
title: "WI-1088 — Runtime activation 连续性与当下 baseline 恢复"
description: "严格 activation 连续性及当下 baseline 恢复的已审实现与证据。"
audience:
  - maintainer
  - reviewer
authority: explicit-user-authorization
workItemId: WI-1088-runtime-activation-amendment-continuity-recovery
status: in_progress
lastVerifiedBy: WI-1088-runtime-activation-amendment-continuity-recovery
---

# WI-1088-runtime-activation-amendment-continuity-recovery — Runtime activation 连续性与当下恢复

human:Ray 在12:02:13Z 对当前13路径 Contract sha256:2396f1f19893040061798a298976e9be9d1c570825c13ba313c5124c024c4976 的真实 confirm_review 已由 agent:Codex 原生记录。checkpoint1 与此前请求均保留，旧6e93确认未复用。当前实现、验收及精确 HEAD 的 CI 证据见 Draft PR1047。

保留真实前向 A/B/owner/Runtime/journal 阶段证据。历史恢复仅当下接受精确 baseline，保留历史未知，仅开启新 preflight。B 曾被审或 CSV 都不是通用豁免。

当前原生范围为原12个 Rust 路径加下列既有 RPC discovery 兼容测试、三语 commands、本 WI 治理/brief，以及三语 reference-parity 中仅本 WI 的行。

- `crates/cockpit-repository/src/contract_amendment.rs`
- `crates/cockpit-repository/tests/contract_amendment.rs`
- `crates/cockpit-cli/tests/contract_amendment_processes.rs`
- `crates/cockpit-repository/src/lifecycle.rs`
- `crates/cockpit-protocol/src/lib.rs`
- `crates/cockpit-repository/src/status_projection.rs`
- `crates/cockpit-repository/src/lib.rs`
- `crates/cockpit-cli/src/main.rs`
- `crates/cockpit-mcp/src/lib.rs`
- `crates/cockpit-repository/tests/lifecycle_entry.rs`
- `crates/cockpit-mcp/tests/material_review.rs`
- `crates/cockpit-mcp/tests/usage.rs`
- `crates/cockpit-mcp/tests/rpc.rs`

真实人工门禁后先临时仓库 RED，再修复、负测和全验证。不操作 WI-1087 真实恢复，不合并、关闭或删除。

Plan: `../../.ai/evidence/WI-1088-runtime-activation-amendment-continuity-recovery/plan-v2-20261017/implementation-plan.v2.md`. Source: `../../.ai/evidence/WI-1088-runtime-activation-amendment-continuity-recovery/plan-v2-20261017/technical-approval-source.json`.
Scope addendum: `../../.ai/evidence/WI-1088-runtime-activation-amendment-continuity-recovery/scope13-20261017/implementation-addendum.md`; technical source: `../../.ai/evidence/WI-1088-runtime-activation-amendment-continuity-recovery/scope13-20261017/technical-approval-source.json`.

执行台账保留原生33节点路线、聚焦结果、此前全部 RED、源码绑定及 CI 诊断的精确实际记录。当前验收与精确 HEAD 的 CI 见 Draft PR1047。软件证据不表示 Runtime 已交付、WI-1087 已实际恢复或生命周期已完成。
