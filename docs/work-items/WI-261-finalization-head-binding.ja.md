---
author: AI Cockpit maintainers
title: "WI-261 — Finalization head binding"
workItemId: WI-261-finalization-head-binding
description: "非 governance drift 後の stale な pre-merge finalization receipt を拒否します。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
---

# WI-261 — Finalization head binding

## Intent

pre-merge finalization evidence を実際の reviewed branch または pull-request head に
binding します。receipt の head fields が自己整合しているだけで、未 review の code を
含む後続 checkout を認可してはなりません。

## Scope

- feature checkout（`HEAD`）または synthetic pull-request merge checkout（reviewed
  feature parent）から reviewed head を解決します。
- exact head、または同じ Work Item に限定した明示的な append-only governance range だけ
  を受理し、後続の code と無関係な drift は拒否します。
- binding と finalization 後の drift を deterministic fixture と shell regression で検証します。
- 英語・中国語・日本語の文書を同期します。

fixture builder は canonical finalization receipt を append-only commit としてモデル化する
ためだけに含まれ、Runtime や Rust crates は変更しません。

## Out of scope

Rust crates、provider 設定、global Agent/MCP 設定、および独立した post-merge
`stale_awaiting_merge_close` recovery lifecycle。

## Acceptance

1. 古い checkout に binding された feature finalization receipt は、後続の code commit
   後に fail-closed になります。
2. synthetic pull-request merge checkout は reviewed feature parent に binding されます。
3. canonical/digest-suffixed finalization、同じ Work Item の close、固定された
   post-finalize evidence append は明示的に限定されます。
4. modified、deleted、renamed、無関係、malformed、非 governance path は拒否されます。
5. 三言語の reference 文書が同じ境界を説明します。

## Verification

- `bash tests/ci/governance_integrity_gate_test.sh`
- `python3 -m py_compile tests/ci/governance_integrity_gate.py tests/ci/fixtures/governance-integrity/build_fixture.py`
- focused gate 通過後、Contract が宣言する workspace verification を実行します。
