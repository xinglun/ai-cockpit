---
author: AI Cockpit maintainers
title: "WI-334 — Evidence Binding と reuse の基礎"
workItemId: WI-334-evidence-binding-reuse
description: "Pinned Evidence Binding/Reuse source を比較し、Python/V1 wire を copy せず Rust semantic counterpart を記録する。"
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-334-evidence-binding-reuse
capabilityClaims:
  - reference_parity
  - evidence_reuse
---

# WI-334 — Evidence Binding と reuse の基礎

## Intent と境界

この Work Item は pinned source revision
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` の 10 path を一つずつ確認します。Target には
Rust-native な複合 evidence model があるため、source Python module や JSON wire を copy せず、
semantic responsibility parity を記録します。

## File-by-file の決定

10 path はすべて `implemented-different-by-design` です。

| Pinned source path | Rust counterpart | 決定 |
| --- | --- | --- |
| `docs/reference/content-bound-evidence-reuse.md` | `crates/cockpit-evidence/src/lib.rs`、`tests/reuse.rs` | content identity は exact composite binding の一部で、reuse は advisory です。 |
| `docs/reference/diff-bound-evidence-reuse.md` | `crates/cockpit-evidence/src/lib.rs`、`crates/cockpit-git/src/lib.rs` | base/head と changed-path identity の mismatch は rerun です。 |
| `docs/reference/environment-bound-reuse.md` | `crates/cockpit-evidence/src/lib.rs`、`crates/cockpit-verification/src/lib.rs` | environment/toolchain/Runtime/profile を明示的に bind し、process environment 全体は serialize しません。 |
| `docs/reference/evidence-binding-foundation.md` | `crates/cockpit-evidence/src/lib.rs`、`crates/cockpit-repository/src/lib.rs` | versioned receipt を strict に検証し、governance/protected check を bypass しません。 |
| `scripts/ai_evidence_binding.py` | `crates/cockpit-evidence/src/lib.rs` | typed struct と content-addressed receipt ID が Python API を置き換えます。 |
| `scripts/ai_diff_bound_reuse.py` | `crates/cockpit-evidence/src/lib.rs`、`crates/cockpit-git/src/lib.rs` | typed diff identity が Python helper を置き換えます。 |
| `scripts/ai_environment_reuse.py` | `crates/cockpit-evidence/src/lib.rs`、`crates/cockpit-verification/src/lib.rs` | bounded な明示 input を使い、credential を読みません。 |
| `tests/test_ai_evidence_binding.py` | `crates/cockpit-evidence/tests/reuse.rs`、`crates/cockpit-repository/tests/receipt_store.rs` | strict schema、tamper、expiry、mismatch、failed/protected、rerun を Rust で検証します。 |
| `tests/test_ai_diff_bound_reuse.py` | `crates/cockpit-evidence/tests/reuse.rs`、`crates/cockpit-git/tests/repository.rs` | clean/changed path、canonical ordering、malformed path、policy mismatch を検証します。 |
| `tests/test_ai_environment_reuse.py` | `crates/cockpit-evidence/tests/reuse.rs`、`crates/cockpit-verification/tests/execution.rs` | environment/toolchain identity、stale/unknown receipt、protected execution を検証します。 |

governance、coverage、security、required-check gate は caller の責任です。exact fresh reuse 以外は再実行し、
source participant、Python、Make、V1 artifact は導入しません。

## Acceptance

- Inventory に WI-334 の 10 record があり、deferred/migrate-gap が残りません。
- 三言語 comparison と parity ledger が同じ semantic/non-wire boundary を示します。
- Rust evidence/reuse test と documentation/inventory check が通過します。
- installed Runtime で identity-bound verification evidence を生成し、reviewed PR、close、正確な cleanup を完了します。

[English](WI-334-evidence-binding-reuse.md) · [简体中文](WI-334-evidence-binding-reuse.zh-CN.md)
