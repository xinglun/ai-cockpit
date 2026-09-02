---
author: AI Cockpit maintainers
title: "WI-506 — v0.2.60 release と public adopter acceptance"
description: "次の identity-bound Runtime release を公開し、reference parity 再開前に public artifact を検証します。"
audience:
  - adopter
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-506-release-v0-2-60
workItemId: WI-506-release-v0-2-60
---

# WI-506 — v0.2.60 release と public adopter acceptance

[English](WI-506-release-v0-2-60.md) · [简体中文](WI-506-release-v0-2-60.zh-CN.md)

## Intent

review 済み main から v0.2.60 を公開し、reference parity を再開する前に隔離した
adopter acceptance で immutable な release artifact を証明します。

## Scope

- 過去の事実を書き換えず、workspace package、lockfile、現在の三言語 release/versioning 文書を
  v0.2.60 に揃えます。
- 三言語 reference-parity ledger にこの Release Work Item を登録します。
- review 済み hosted PR を default branch に同期した後、annotated tag を作成し、archive、
  checksum、SBOM、provenance、manifest を公開します。
- downloaded immutable artifact だけで public adopter と N-1 acceptance を実行し、isolation、
  evidence binding、`not_ready` scaffold、temporary-root cleanup を証明します。
- 公開 binary を本 repository に install し、inspect、status、doctor、Agent doctor、
  documentation promotion の健康状態を確認します。

## Out of scope

local reference source、object/adopter repository、global Agent/MCP/Homebrew configuration、
source/workspace binary fallback、無関係な Runtime redesign、生成 governance record の手編集。

## Acceptance criteria

1. workspace package と lockfile が v0.2.60 を示し、現在の三言語 release guidance が過去の履歴を
   書き換えず更新されること。
2. 同期済み default branch から annotated v0.2.60 tag を作成する前に reviewed PR の hosted checks が
   すべて成功すること。
3. public Release が identity-bound archive、SHA256 checksum、SBOM、provenance、release manifest を提供すること。
4. public adopter と N-1 acceptance が downloaded immutable artifact のみを使い、isolation と temporary-root
   cleanup を証明し、`first-adopter-smoke=not_ready` を維持すること。
5. 公開 binary を本 repository に install 後、inspect、status、doctor、Agent doctor、post-close 文書チェックが健康であること。
6. この Work Item に可視の human Outcome、archive、finalization、close、正確な branch/worktree cleanup が記録されること。

## Verification

```text
cargo test --locked --workspace
```

Release publication と public acceptance は post-release evidence です。失敗した publication は immutable な履歴として保持し、成功扱いに再標識または再利用しません。

## Boundary

Runtime binary は共有されますが、本 repository の Protocol、Work Items、evidence、knowledge、adapter は repository-local です。
Runtime の公開が他 repository を暗黙に attach または変更することはありません。
