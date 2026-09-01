---
author: AI Cockpit maintainers
title: "WI-478 — v0.2.57 release と公開 adopter acceptance"
description: "v0.2.56 の公開失敗後、修正した順序で Runtime を公開し、公開 artifact を隔離環境で検証する。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-478-release-v0-2-57
terminalArchive: .ai/work-items/archive/WI-478-release-v0-2-57.contract.json
terminalVerification: .ai/evidence/WI-478-release-v0-2-57.verification.json
terminalFinalization: .ai/decisions/WI-478-release-v0-2-57.finalize.json
terminalDecision: .ai/decisions/WI-478-release-v0-2-57.close.json
workItemId: WI-478-release-v0-2-57
---

# WI-478 — v0.2.57 release と公開 adopter acceptance

[English](WI-478-release-v0-2-57.md) · [简体中文](WI-478-release-v0-2-57.zh-CN.md)

## Intent

v0.2.56 の公開失敗後、修正した順序で新しい immutable Runtime Release を公開します。公開 binary が隔離した adopter でゼロから利用でき、その後この repository に install できることを示します。local reference source や adopter repository は変更しません。

## Scope

- workspace package、lockfile、現在の三言語 release/versioning 文書を `v0.2.57` に揃え、失敗履歴と historical evidence を保持する。
- 三言語 reference-parity ledger にこの Work Item を登録する。
- reviewed hosted PR の checks を通過してから annotated tag を作り、archive、checksum、SBOM、provenance、manifest、Runtime identity を公開する。
- 公開 download artifact だけで adopter/N-1 acceptance を実行し、forbidden-write、evidence binding、temporary-root cleanup を検証する。
- 公開 binary をこの repository に install し、inspect/status/doctor/Agent doctor/ready-on-base を確認する。
- tag 前に verification、human Outcome、archive、resource finalization、close、文書 promotion、branch/worktree cleanup を完了する。

## Out of scope

local reference source、`/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator`、他の adopter、global Agent/MCP/Homebrew configuration、source/workspace binary fallback、無関係な Runtime redesign、生成済み status/evidence/receipt/archive/decision の手編集。

## Acceptance criteria

1. workspace package、lockfile、必須の三言語 release 文書が、過去の事実を書き換えず `v0.2.57` を示す。
2. reviewed PR の hosted checks が通過してから merge し、annotated `v0.2.57` tag は同期済み reviewed default branch を正確に指し、Work Item close 後だけ作成する。
3. 公開 Release が archive、SHA256、SBOM、provenance、identity-bound manifest を提供する。
4. adopter/N-1 は immutable な公開 artifact だけを使い、`first-adopter-smoke=not_ready` を保持し、repository/runtime identity、隔離、成功/失敗時の temporary-root cleanup を証明する。
5. 公開 binary をこの repository に install し、inspect/status/doctor/Agent doctor と文書 promotion で healthy attachment と readiness を確認する。
6. 必須の `🟢`/`🟡`/`🔴` marker を持つ visible human Outcome を出し、archive/finalization/close と正確な cleanup を完了する。

## Verification

```text
cargo test --locked --workspace
```

公開と公開 acceptance は post-release evidence です。失敗した公開は immutable な失敗履歴として保持し、成功に書き換えたり再利用したりしません。

## Boundary

Runtime binary は共有されますが、この repository の Protocol、Work Item、evidence、knowledge、adapter は private に保持します。Runtime の公開は target repository を暗黙に attach/変更しません。
