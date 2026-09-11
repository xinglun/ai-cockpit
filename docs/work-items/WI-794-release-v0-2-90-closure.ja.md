---
author: AI Cockpit maintainers
title: "WI-794 — v0.2.90 governed release closure"
description: "公開済み v0.2.90 artifact の post-release closure successor。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized-release-recovery
workItemId: WI-794-release-v0-2-90-closure
lastVerifiedBy: WI-794-release-v0-2-90-closure
terminalArchive: .ai/work-items/archive/WI-794-release-v0-2-90-closure.contract.json
terminalVerification: .ai/evidence/WI-794-release-v0-2-90-closure.verification.json
terminalFinalization: .ai/decisions/WI-794-release-v0-2-90-closure.finalize.json
terminalDecision: .ai/decisions/WI-794-release-v0-2-90-closure.close.json
---

[English](WI-794-release-v0-2-90-closure.md) · [简体中文](WI-794-release-v0-2-90-closure.zh-CN.md)

# WI-794 — v0.2.90 governed release closure

WI-794 は WI-793 の append-only successor です。WI-793 は歴史的な blocked evidence として保持します。WI-793 の required closure scenario は、その scenario が許可を要求する `finish` transition の後でなければ証明できなかったためです。Runtime は実際の `finish.governance` stop を記録しており、この Work Item はそれを書き換えません。

## Closure boundary

v0.2.90 Release は immutable で、merged main commit
`2d45d6f2e0c99131c6476bd1aa3b3dedc5d81921` に bind されています。公開インストールと N-1 upgrade の receipt は `.ai/evidence/external/v0.2.90/` に保持されています。

この Work Item は finish 前に実行可能な事実を検証し、その後 Runtime lifecycle と明示的な post-merge finalization gate により archive、close、正確な branch/worktree cleanup、synchronized main を完了します。Rust、Runtime、release asset、tag、predecessor evidence は変更しません。

## Verification

- `cargo test --locked --workspace`
- 公開 v0.2.90 adopter acceptance と isolation/cleanup receipt
- 公開 v0.2.89 → v0.2.90 upgrade receipt と byte-identical history digest
- Runtime status/validate と最終 `promote_closed_work_item.py --check-all`

## Evidence policy

Release と adopter の receipt は外部 immutable facts です。Runtime-owned の Contract、Summary、Outcome、archive、finalization、close record はインストール済み Runtime が生成します。transition が実行される前に将来の closure state を verified として投影しません。
