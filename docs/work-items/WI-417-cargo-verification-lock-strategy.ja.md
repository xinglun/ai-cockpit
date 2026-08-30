---
author: AI Cockpit maintainers
title: WI-417 — 決定的な Cargo 検証 scaffold の command 選択
description: repository の事実から実行可能な既定 Cargo 検証 command を選択する。
workItemId: WI-417-cargo-verification-lock-strategy
audience: [adopter, contributor, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-417-cargo-verification-lock-strategy
---

# WI-417 — 決定的な Cargo 検証 scaffold の command 選択

[English](WI-417-cargo-verification-lock-strategy.md) · [简体中文](WI-417-cargo-verification-lock-strategy.zh-CN.md)

## Intent

Cargo Work Item scaffold の activation 時に生成する既定検証 command を、対象
repository で実行可能にする。追跡された `Cargo.lock` があれば
`cargo test --locked --workspace`、lockfile がない Cargo repository では
`cargo test --workspace` を選び、非 Cargo repository には Cargo command を捏造しない。

## Scope と boundary

`start` と recovery scaffold の activation は同じ決定的な規則を使う。この Work
Item は command 選択と reference documentation のみを変更し、検証 semantics、
release/adopter harness、Sentinel source、global Agent/MCP configuration は変更しない。

## Evidence

- Archive: `.ai/work-items/archive/WI-417-cargo-verification-lock-strategy.contract.json`
- Verification: `.ai/evidence/WI-417-cargo-verification-lock-strategy.verification.json`
- Finalization: `.ai/decisions/WI-417-cargo-verification-lock-strategy.finalize.json`
- Close: `.ai/decisions/WI-417-cargo-verification-lock-strategy.close.json`
- Review 済み PR: [#382](https://github.com/xinglun/ai-cockpit/pull/382)

lockfile、有無、非 Cargo の各 targeted test と、Runtime v0.2.43 下の full locked
workspace test suite が pass した。
