---
author: AI Cockpit maintainers
title: "WI-459 — v0.2.53 release と public binary acceptance"
workItemId: WI-459-release-v0-2-53
description: "レビュー済み Rust Runtime patch を公開し、adopter baseline で public binary を検証する。"
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-459-release-v0-2-53
---

# WI-459 — v0.2.53 release と public binary acceptance

この Work Item は default branch のレビュー済み変更を v0.2.53 として公開し、
公開後は public binary だけを使って adopter acceptance を実行した後、reference
source の逐次 parity queue に戻るためのものです。

[English](WI-459-release-v0-2-53.md) · [简体中文](WI-459-release-v0-2-53.zh-CN.md)

## Scope

- workspace package と lockfile の identity を v0.2.53 にそろえる。
- 三言語の installation、release、versioning projection を更新し、過去の release/failure history は保持する。
- annotated tag、manifest、checksum、SBOM、provenance、staged/public adopter gate を release authority として維持する。
- merge 後に public v0.2.53 binary を download して検証し、Runtime、repository、isolation、cleanup、lifecycle receipt を保持する。

## Out of scope

WI-445 が保持する reference inventory/parity ledger、local reference checkout、object repository、
global Agent/MCP configuration、Homebrew tap の変更、source-build fallback、無関係な Runtime behavior。

## Acceptance

- workspace metadata、lockfile、三言語の release document が v0.2.53 を示し、過去の release truth を書き換えない。
- レビュー済み PR と hosted workflow が annotated tag、source commit、manifest、Cargo.lock digest、
  archive/SBOM checksum、provenance、public asset を binding する。
- version、workflow、documentation、workspace quality gate が source fallback なしで成功する。
- 公開後 adopter harness が v0.2.53 archive を download/checksum verify し、repository/runtime identity、
  isolation、evidence reuse、cleanup、`first-adopter-smoke` の `not_ready` contract を receipt に残す。
- 公開と acceptance 後に default branch を同期し、Work Item を close して `ready_on_base` にする。

## Verification

- `bash tests/release/version_consistency.sh --repo <repo>`
- `bash tests/release/version_consistency_test.sh`
- `bash tests/release/workflow_policy.sh .github/workflows/release.yml`
- strict repository gate manifest と documentation acceptance
- `cargo test --locked --workspace`
- public v0.2.53 だけを使う post-release `tests/release/adopter_acceptance.sh`

## Release boundary

レビュー済み merge と default branch 同期の後だけ annotated tag を push します。provider Release は
source、artifact、staged-adopter gate がすべて成功した後に workflow が作成します。public acceptance は
publication 後だけ実行し、失敗しても公開済み Release の truth は書き換えません。
