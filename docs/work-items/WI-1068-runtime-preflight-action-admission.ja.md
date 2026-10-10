---
author: AI Cockpit maintainers
title: "WI-1068 — Runtime preflight action admission"
description: "現在の Work Item Contract と repository snapshot に結び付いた green preflight の後にだけ finish を許可します。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1068-runtime-preflight-action-admission
status: implemented
authority: historical-projection
lastVerifiedBy: WI-1068-runtime-preflight-action-admission
terminalArchive: .ai/work-items/archive/WI-1068-runtime-preflight-action-admission.contract.json
terminalVerification: .ai/evidence/WI-1068-runtime-preflight-action-admission.verification.json
terminalDecision: .ai/decisions/WI-1068-runtime-preflight-action-admission.close.json
---

# WI-1068 — Runtime preflight action admission

このページはアーカイブ済み Work Item の文書投影です。Lifecycle、verification、
close の不変記録は `.ai/work-items/archive/WI-1068-runtime-preflight-action-admission.*`
を正とします。

## 目的と動作

この Work Item は Status projection を `finish` の green preflight 要件に合わせました。
verification evidence が有効でも、preflight が未実行、green 以外、または古い
Contract / repository snapshot に結び付いている場合、Runtime は `run_preflight`
を提示し、`finish` を許可しません。現在の Contract と repository snapshot の両方に
結び付いた preflight が green になった後にだけ `finish` が許可されます。既存の
verification、governance-control、human-review の確認も引き続き適用されます。

実装対象は `crates/cockpit-repository/src/status_projection.rs` とそのテスト、
`crates/cockpit-cli/tests/collaboration_consistency.rs`、および `.ai/policy.json` です。
テスト fixture では `directory.path()` を `&Path` として明示的に束縛してから
path を結合し、assessment が `repository_material_inspection_unavailable` を誤報しない
ようにしました。

## アーカイブ結果

アーカイブ Contract は `cargo fmt --all -- --check`、
`cargo test --locked --workspace`、workspace Clippy を宣言しています。アーカイブ
Outcome は verification を `verified` と記録しています。Work Item owner が
宣言していないため、task report の human-visible benefit は unknown のままです。

[English](WI-1068-runtime-preflight-action-admission.md) ·
[简体中文](WI-1068-runtime-preflight-action-admission.zh-CN.md)
