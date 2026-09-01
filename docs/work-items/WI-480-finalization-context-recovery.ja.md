---
author: AI Cockpit maintainers
title: "WI-480 — finalization context recovery gate"
description: "終端 lifecycle の前に曖昧な provisional resource context を拒否します。"
audience:
  - maintainer
  - reviewer
  - adopter
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-480-finalization-context-recovery
workItemId: WI-480-finalization-context-recovery
---

# WI-480 — finalization context recovery gate

この Runtime 変更では、裸の `pending` provider sentinel を既存の
`pending:<stable-reference>` と同じ provisional context として扱います。
実際のレビュー済み resource を明示的な `finalize-plan` で束縛するまで
`finish` と `archive` は fail-closed です。WI-479 の immutable record は
append-only successor path で recovery し、書き換えません。

[English](WI-480-finalization-context-recovery.md) · [简体中文](WI-480-finalization-context-recovery.zh-CN.md)

## Scope

- exact `pending` sentinel を provisional に分類する;
- finish の拒否と recoverability を protocol/lifecycle regression で検証する;
- 三言語で明示的な finalization boundary を文書化する。

## Out of scope

Release、adopter repository、foreign Runtime policy、WI-479 の Contract、evidence、archive、Outcome、event、recovery bytes の書き換え。

## Verification

- `cargo test --locked -p cockpit-protocol --test resource_finalization`
- `cargo test --locked -p cockpit-repository --test archive_integrity`
- `cargo test --locked --workspace`
- `cargo fmt --all -- --check`
