---
author: AI Cockpit maintainers
title: "WI-503 — v0.2.59 release と public adopter acceptance"
description: "次の identity-bound Runtime release を公開し、reference parity 再開前に public artifact を検証する。"
audience:
  - adopter
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-503-release-v0-2-59
workItemId: WI-503-release-v0-2-59
---

# WI-503 — v0.2.59 release と public adopter acceptance

[English](WI-503-release-v0-2-59.md) · [简体中文](WI-503-release-v0-2-59.zh-CN.md)

## Intent

WI-502 の terminal documentation gate fix 後に immutable な `v0.2.59` Runtime
release を公開します。public binary が isolated adopter でゼロから動作し、N-1
acceptance を通過し、次の reference source file comparison の前に本 repository
へ install されることを確認します。

## Scope

- 過去の release fact を書き換えず、workspace package、lockfile、現在の三言語
  release/versioning guidance を `v0.2.59` に合わせます。
- 三言語の reference-parity ledger にこの release Work Item を登録します。
- synchronized `main` から annotated tag を作る前に reviewed hosted PR を通過し、
  archive、checksum、SBOM、provenance、manifest を公開します。
- downloaded immutable artifact だけで public adopter と N-1 acceptance を実行し、
  isolation、evidence binding、`not_ready` scaffold、temporary-root cleanup を証明します。
- 公開 binary を本 repository に install し、inspect、status、doctor、Agent doctor、
  documentation-promotion の健全性を確認します。

## Out of scope

local reference source、object/adopter repository、global Agent/MCP または Homebrew
設定、source/workspace binary fallback、無関係な Runtime redesign、generated
governance record の手編集。

## Acceptance criteria

1. workspace package、`Cargo.lock`、現在の三言語 release document が `v0.2.59` を示し、
   過去の release history を書き換えない。
2. synchronized default branch から annotated `v0.2.59` tag を作成する前に reviewed
   PR の hosted checks がすべて成功する。
3. public Release が identity-bound archive、SHA256 checksum、SBOM、provenance、
   release manifest を公開する。
4. public adopter と N-1 acceptance は downloaded immutable artifact のみを使い、
   isolation と temporary-root cleanup を証明し、`first-adopter-smoke=not_ready` を保持する。
5. 公開 binary を本 repository に install 後、inspect、status、doctor、Agent doctor、
   post-close documentation check が正常である。
6. この Work Item が公開前に visible human Outcome、archive、finalization、close、
   branch/worktree の正確な cleanup を持つ。

## Verification

```text
cargo test --locked --workspace
```

Release publication と public acceptance は post-release evidence です。失敗した
publication は immutable な failed history として保持し、成功に再標識したり再利用しません。

## Boundary

Runtime binary は共有しますが、本 repository の Protocol、Work Item、evidence、knowledge、
adapter は repository-local です。Runtime の公開は他 repository を暗黙に attach/変更しません。
