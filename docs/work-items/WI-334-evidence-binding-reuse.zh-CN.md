---
author: AI Cockpit maintainers
title: "WI-334——Evidence Binding 与 reuse 基础"
workItemId: WI-334-evidence-binding-reuse
description: "比对固定 Evidence Binding/Reuse 参考源，并记录 Rust 语义对应物，不复制 Python/V1 wire。"
audience:
  - adopter
  - maintainer
  - reviewer
status: recovered
authority: canonical
lastVerifiedBy: WI-334-evidence-binding-reuse
capabilityClaims:
  - reference_parity
  - evidence_reuse
---

# WI-334——Evidence Binding 与 reuse 基础

## 意图与边界

本 Work Item 逐一读取固定源版本
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` 中的 10 个路径。目标已有 Rust 原生组合 evidence
模型，因此本批记录语义责任对齐，不复制源 Python 模块或 JSON wire。

## 逐文件决定

10 个路径全部为 `implemented-different-by-design`：

| 固定源路径 | Rust 对应物 | 决定 |
| --- | --- | --- |
| `docs/reference/content-bound-evidence-reuse.md` | `crates/cockpit-evidence/src/lib.rs`、`tests/reuse.rs` | content identity 是精确组合绑定的一部分；reuse 仍只是 advisory。 |
| `docs/reference/diff-bound-evidence-reuse.md` | `crates/cockpit-evidence/src/lib.rs`、`crates/cockpit-git/src/lib.rs` | base/head 与 changed-path identity 不匹配必须 rerun。 |
| `docs/reference/environment-bound-reuse.md` | `crates/cockpit-evidence/src/lib.rs`、`crates/cockpit-verification/src/lib.rs` | 显式绑定 environment/toolchain/Runtime/profile；不整体序列化进程环境。 |
| `docs/reference/evidence-binding-foundation.md` | `crates/cockpit-evidence/src/lib.rs`、`crates/cockpit-repository/src/lib.rs` | 版本化 receipt 严格校验，不能绕过治理或 protected checks。 |
| `scripts/ai_evidence_binding.py` | `crates/cockpit-evidence/src/lib.rs` | typed structs 和 content-addressed receipt ID 替代 Python API。 |
| `scripts/ai_diff_bound_reuse.py` | `crates/cockpit-evidence/src/lib.rs`、`crates/cockpit-git/src/lib.rs` | typed diff identity 替代 Python helper。 |
| `scripts/ai_environment_reuse.py` | `crates/cockpit-evidence/src/lib.rs`、`crates/cockpit-verification/src/lib.rs` | 显式有界输入替代源 adapter；不读取凭据。 |
| `tests/test_ai_evidence_binding.py` | `crates/cockpit-evidence/tests/reuse.rs`、`crates/cockpit-repository/tests/receipt_store.rs` | strict schema、篡改、expiry、mismatch、failed/protected 和 rerun 场景由 Rust 覆盖。 |
| `tests/test_ai_diff_bound_reuse.py` | `crates/cockpit-evidence/tests/reuse.rs`、`crates/cockpit-git/tests/repository.rs` | clean/changed paths、canonical ordering、非法路径和 policy mismatch 有测试。 |
| `tests/test_ai_environment_reuse.py` | `crates/cockpit-evidence/tests/reuse.rs`、`crates/cockpit-verification/tests/execution.rs` | environment/toolchain identity、stale/unknown receipt 与 protected execution 有测试。 |

治理、coverage、安全和 required-check gate 仍由调用方负责；除非是精确 fresh reuse，其他结果都必须
重新执行。不引入源 participant、Python、Make 或 V1 artifact。

## 验收

- Inventory 正好有 10 条 WI-334 记录，且没有 deferred 或 migrate-gap。
- 三语 comparison 与 parity ledger 说明相同的语义/非 wire 边界。
- Rust evidence/reuse 测试及文档/inventory 检查通过。
- 使用已安装 Runtime 生成绑定 verification evidence，并完成审查 PR、关闭和精确清理。

[English](WI-334-evidence-binding-reuse.md) · [日本語](WI-334-evidence-binding-reuse.ja.md)
