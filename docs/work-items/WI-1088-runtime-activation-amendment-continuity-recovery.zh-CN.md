# WI-1088-runtime-activation-amendment-continuity-recovery — Runtime activation 连续性与当下恢复

技术范围已批准；原生 scope 修订及真实人工门禁待完成。产品验收：not_run。

保留真实前向 A/B/owner/Runtime/journal 阶段证据。历史恢复仅当下接受精确 baseline，保留历史未知，仅开启新 preflight。B 曾被审或 CSV 都不是通用豁免。

已审范围为下列 12 个 Rust 路径、三语 commands、本 WI 治理/三语 brief，以及三语 reference-parity 中仅 WI-1088 的真实阶段行。

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

真实人工门禁后先临时仓库 RED，再修复、负测和全验证。不操作 WI-1087 真实恢复，不合并、关闭或删除。

Plan: `../../.ai/evidence/WI-1088-runtime-activation-amendment-continuity-recovery/plan-v2-20261017/implementation-plan.v2.md`. Source: `../../.ai/evidence/WI-1088-runtime-activation-amendment-continuity-recovery/plan-v2-20261017/technical-approval-source.json`.
