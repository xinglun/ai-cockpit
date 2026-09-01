---
author: AI Cockpit maintainers
title: "WI-463 — reference parity documentation promotion retry"
workItemId: WI-463-reference-parity-promotion-retry
description: "CI governance evidence の順序で停止した前回の不変 delivery を、clean base から再 delivery する。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-463-reference-parity-promotion-retry
terminalArchive: .ai/work-items/archive/WI-463-reference-parity-promotion-retry.contract.json
terminalVerification: .ai/evidence/WI-463-reference-parity-promotion-retry.verification.json
terminalFinalization: .ai/decisions/WI-463-reference-parity-promotion-retry.finalize.json
terminalDecision: .ai/decisions/WI-463-reference-parity-promotion-retry.close.json
---

# WI-463 — reference parity documentation promotion retry

この bounded successor は verified close 済みの WI-461 を reader-facing parity
projection として再 delivery します。変更は documentation のみに限定し、Runtime、
release truth、object repository、immutable evidence は変更しません。失敗した WI-462
delivery は独立した audit record として保持します。

[English](WI-463-reference-parity-promotion-retry.md) · [简体中文](WI-463-reference-parity-promotion-retry.zh-CN.md)

## Scope

- 三言語の parity ledger で WI-461 を terminal Implemented に昇格する。
- WI-461 の immutable archive、verification、finalization、close path を保持する。
- この Work Item 自身の三言語 page と archive 前の parity row を維持する。

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh <repo>`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
- `cargo test --locked --workspace`

レビュー済み merge、finalization、close の後に documentation-promotion helper が
この Work Item の terminal field を昇格します。
