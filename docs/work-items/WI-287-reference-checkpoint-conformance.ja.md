---
author: AI Cockpit maintainers
title: "WI-287 — Reference checkpoint conformance"
workItemId: WI-287-reference-checkpoint-conformance
description: "Rust-native fail-closed regression と truthful ledger で reference checkpoint の比較 gap を閉じる。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-287-reference-checkpoint-conformance
authority: canonical
---

# WI-287 — Reference checkpoint conformance

## 目的

この bounded batch は、reference checkpoint のファイル単位比較に残る台帳の
穴を閉じ、Rust-native semantics を証明します。Python、Make、YAML、V1 の
wire format は Runtime にコピーしません。

## 比較したファイル

| Reference file | Rust counterpart | 境界 |
| --- | --- | --- |
| `scripts/ai_checkpoint.py` | typed `CheckpointPolicy`/`CheckpointEvidence`、repository checkpoint/amendment validator | semantic parity のみ。直接 JSON-wire 互換を主張しない |
| `tests/test_ai_checkpoint.py` | `agent_risk_checkpoint.rs`、`lifecycle_order.rs` | ordering、resume stale、amendment lineage、immutable evidence の Rust regression |
| `tests/test_outcome_lifecycle_rules.py` | `agent_rule_parity.rs`、`AGENTS.md`、`.ai/README.md`、`docs/reference/agent-workflow.md` | project-native な Agent rule projection。template の copy ではない |

## 変更

- verification result が存在した後の `before_edit` checkpoint を fail-closed にする。
- 最新 `resumeHistory.recordedAt` が不正なら、欠落として扱わず拒否する。
- current Work Item repair、visible Outcome terminality、narrow successor の静的 parity 断言を強化する。
- checkpoint の二つの source file を `implemented-different-by-design` として台帳に登録する。

## Object/adopter 境界

これらは Runtime と repository protocol の挙動なので、新しい adopter にも同じ
制御が継承されます。全コマンドは明示的な repository context を要求し、unknown
を表示したまま human Outcome を handoff 境界とします。

## 検証

`cargo test --locked --workspace`、conformance ledger regression、documentation
acceptance、repository governance gate を実行します。
