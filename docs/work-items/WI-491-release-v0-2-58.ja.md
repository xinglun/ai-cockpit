---
author: AI Cockpit maintainers
title: "WI-491 — v0.2.58 release と公開 adopter acceptance"
description: "次の identity-bound Runtime を公開し、reference parity を再開する前に公開 artifact を検証する。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-491-release-v0-2-58
terminalArchive: .ai/work-items/archive/WI-491-release-v0-2-58.contract.json
terminalVerification: .ai/evidence/WI-491-release-v0-2-58.verification.json
terminalFinalization: .ai/decisions/WI-491-release-v0-2-58.finalize.json
terminalDecision: .ai/decisions/WI-491-release-v0-2-58.close.json
workItemId: WI-491-release-v0-2-58
---

# WI-491 — v0.2.58 release と公開 adopter acceptance

[English](WI-491-release-v0-2-58.md) · [简体中文](WI-491-release-v0-2-58.zh-CN.md)

## Intent

WI-490 の terminal documentation gate 修正後に identity-bound な `v0.2.58`
Runtime Release を公開します。公開 binary が隔離 adopter でゼロから動作し、
N-1 acceptance を通過してこの repository に install できることを確認してから、
次の reference source の逐ファイル比較を再開します。

## Scope

- workspace package、lockfile、現在の三言語 release/versioning 文書を `v0.2.58` に揃え、過去の事実を保持する。
- 三言語 reference-parity ledger にこの release Work Item を登録する。
- reviewed hosted PR の checks 後に同期済み `main` から annotated tag を作り、archive、checksum、SBOM、provenance、manifest を公開する。
- download した immutable artifact だけで公開 adopter/N-1 acceptance を実行し、隔離、evidence binding、`not_ready` scaffold、cleanup を検証する。
- 公開 binary をこの repository に install し、inspect/status/doctor/Agent doctor と文書 promotion を確認する。

## Out of scope

local reference source、object/adopter repository、global Agent/MCP/Homebrew
configuration、source/workspace binary fallback、無関係な Runtime redesign、
生成済み governance record の手編集。

## Acceptance criteria

1. workspace package、`Cargo.lock`、現在の三言語 release 文書が、過去の release facts を書き換えず `v0.2.58` を示す。
2. reviewed PR の hosted checks 通過後に merge し、annotated `v0.2.58` tag は同期済み reviewed default branch を正確に指す。
3. 公開 Release が identity-bound archive、SHA256、SBOM、provenance、release manifest を提供する。
4. 公開 adopter/N-1 は immutable な download artifact だけを使い、隔離と temporary-root cleanup を証明し、`first-adopter-smoke=not_ready` を保持する。
5. 公開 binary をこの repository に install した後も inspect/status/doctor/Agent doctor と post-close 文書検査が healthy である。
6. 公開前に visible human Outcome、archive、finalization、close、正確な branch/worktree cleanup を完了する。

## Verification

```text
cargo test --locked --workspace
```

公開と公開 acceptance は post-release evidence です。失敗した公開は immutable な
失敗履歴として保持し、成功に書き換えたり再利用したりしません。

## Boundary

Runtime binary は共有されますが、この repository の Protocol、Work Item、evidence、
knowledge、adapter は repository-local に保持します。Runtime の公開は他の repository
を暗黙に attach/変更しません。
