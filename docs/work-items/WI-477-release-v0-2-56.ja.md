---
author: AI Cockpit maintainers
title: "WI-477 — v0.2.56 release と公開 adopter acceptance"
description: "review 済み Runtime patch を公開し、adopter repository を変更せず immutable artifact を受け入れる。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implementation_active
authority: canonical
lastVerifiedBy: WI-477-release-v0-2-56
workItemId: WI-477-release-v0-2-56
---

# WI-477 — v0.2.56 release と公開 adopter acceptance

## Intent

次の review 済み Runtime patch を公開し、immutable な公開 binary が隔離された
adopter を治理できることを確認する。本 repository に install した後、local
reference source の file-by-file 比較へ戻る。reference source や adopter
repository は変更しない。

## Scope

- workspace package identity と三言語の release/versioning guidance を `v0.2.56` に揃え、過去の事実を保持する。
- archive 前にこの Work Item を三言語 parity ledger に登録する。
- review 済み PR、annotated tag、manifest、checksum、SBOM、provenance、artifact identity evidence を保持する。
- download した公開 Release artifact だけで public adopter と N-1 acceptance を隔離 root で実行し、evidence reuse と一時 root cleanup を確認する。
- 公開 binary を本 repository に install/upgrade し、repository、Runtime、Agent、readiness を確認する。

## Out of scope

local reference source、`/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator`、他の adopter、global Agent/MCP 設定、Homebrew tap 変更、source fallback、無関係な Runtime architecture 変更。

## Acceptance criteria

1. workspace version、lockfile、必要な三言語 release 文書が `v0.2.56` を示し、history を rewrite しない。
2. PR は merge 前に hosted checks を通過し、annotated `v0.2.56` tag は同期済み review main commit を指す。
3. 公開 Release は archives、checksum、SBOM/provenance、identity-bound manifest を提供する。
4. public adopter/N-1 acceptance は immutable artifact のみを使い、`first-adopter-smoke=not_ready`、identity/digest、isolation、成功/失敗時 cleanup を証明する。
5. 公開 binary を本 repository に install 後、`inspect`/`status`/`doctor`/`agent doctor` が healthy `ready_on_base` を示す。
6. visible Human Outcome、archive/finalization/close、documentation promotion、正確な branch/worktree cleanup を完了する。

## Verification

```text
cargo test --locked --workspace
```

Release publication と public acceptance は post-release evidence であり、失敗時に既存の Release truth を書き換えない。

## Boundary

Runtime upgrade は共有 executable だけを置き換え、Repository Protocol state は repository 内に保持する。公開は repository を暗黙に attach/変更しない。
