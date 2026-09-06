---
author: AI Cockpit maintainers
title: "WI-622 — direct-merge recovery plan/apply 契約"
description: "最初の direct-merge recovery plan を二つの CLI apply entry point で実行可能に保つ。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-622-direct-merge-cli-contract
status: in_progress
authority: canonical
lastVerifiedBy: WI-622-direct-merge-cli-contract
---

# WI-622 — direct-merge recovery plan/apply 契約

## Intent

Runtime の first-record direct-merge recovery plan を実行可能な契約にします。完全な
`suggestedReceipt` に `humanInputRequired` の項目だけを補えば、
`work-item finalize-recovery` で記録でき、通常の `work-item finalize` で冪等 replay
できます。PR の捏造や生成された repository record の手編集は行いません。

## Boundary

Rust protocol/repository/CLI の回帰契約と三言語の command/troubleshooting 文書を対象にします。
adopter repository、release/adopter harness、source template implementation、global
Agent/MCP configuration は対象外です。既存の厳格な identity、merge parent、repository、
fail-closed 検証を権威として維持します。

## Acceptance

- 実際の二つの parent を持つ merge から、number `0`、historical provider、merge parents、
  repository identity を含む完全な `direct_merge_no_pr` `suggestedReceipt` を生成する。
- `humanInputRequired` の項目だけを追加すれば、`finalize-recovery` の first write と
  `finalize` の idempotent replay が成功する。
- identity が正しい receipt のみ `finalize-verify`/`close` を許可し、malformed、foreign、
  contradictory input は引き続き拒否する。
- CLI help と三言語の command/troubleshooting 文書が同じ apply 契約と no-hand-edit/
  no-invented-PR 境界を説明する。

## Verification

```text
cargo test --locked -p cockpit-repository --test resource_finalization_transition
cargo test --locked -p cockpit-cli --test resource_finalization
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
bash tests/docs/documentation_acceptance.sh
```

See also: [English](WI-622-direct-merge-cli-contract.md) ·
[中文](WI-622-direct-merge-cli-contract.zh-CN.md)。
