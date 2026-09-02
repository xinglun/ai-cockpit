---
author: AI Cockpit maintainers
title: "WI-515 — v0.2.63 リリースと historical adopter recovery acceptance"
description: "legacy shared-worktree と direct-merge recovery の修正を公開し、不変の adopter evidence を提供する。"
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
workItemId: WI-515-release-v0-2-63
lastVerifiedBy: WI-515-release-v0-2-63
---

[English](WI-515-release-v0-2-63.md) · [简体中文](WI-515-release-v0-2-63.zh-CN.md)

# WI-515 — v0.2.63 リリースと historical adopter recovery acceptance

## Intent

歴史的な shared-primary `retained` finalization と PR なし direct-merge
recovery を正しく扱う Runtime 修正を公開する。object/adopter repository は
read-only のまま、公開 artifact を使って独自に受け入れ確認する。

## Scope

- workspace version と現行三言語の release/versioning guidance を v0.2.63 に合わせ、過去の事実は変更しない。
- 三言語 parity ledger にこの Release Work Item を登録する。
- synchronized main から annotated tag を作る前に、review 済み PR の hosted checks を通す。
- archive、SHA256SUMS、SBOM、provenance、manifest を公開し、immutable download だけで public adopter と N-1 を検証する。
- 公開 binary を本 repository に install し、inspect、status、doctor、Agent doctor、文書 promotion を確認する。

## Out of scope

local reference source、object/adopter repository、global Agent/MCP または Homebrew
設定、source/workspace fallback、無関係な Runtime 変更、generated governance record の手編集。

## Acceptance criteria

1. workspace package と lockfile が v0.2.63 を示し、過去の release facts は保持される。
2. synchronized main から annotated tag を作る前に、review 済み PR の hosted checks が全て成功する。
3. 公開 release の archive、SHA256SUMS、SBOM、provenance、manifest が tag、bytes、digest で一致する。
4. public adopter と N-1 は immutable download のみを使い、isolation と temporary-root cleanup を証明し、
   `first-adopter-smoke=not_ready` を保持する。
5. 公開 binary の install 後も health と close 後の文書 gate が green である。
6. 公開完了前に visible human Outcome、archive、finalization、close、exact cleanup を記録する。

## Verification

```text
cargo test --locked --workspace
```

Release publication と object repository acceptance は別の evidence boundary である。失敗した publication は
immutable history として残し、成功扱いに変更したり再利用したりしない。

## Boundary

Runtime binary は共有されるが、各 repository の Protocol、Contract、evidence、knowledge、adapter state は分離される。
v0.2.63 の公開が他の repository を暗黙に attach または変更することはない。
