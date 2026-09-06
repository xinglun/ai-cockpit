---
title: "WI-607 — v0.2.81 release と adopter acceptance"
description: "次の Rust Runtime patch を公開し、immutable artifact と adopter 境界を検証する。"
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-607-release-v0-2-81
lastVerifiedBy: WI-607-release-v0-2-81
---

[English](WI-607-release-v0-2-81.md) · [简体中文](WI-607-release-v0-2-81.zh-CN.md)

# WI-607 — v0.2.81 release と adopter acceptance

## 目的

レビュー済み Rust Runtime を `v0.2.81` として公開し、immutable な公開 Release
artifact だけで install と upgrade の境界を検証する。本版は GitHub Release API
認証修正を含み、source template の移植ではない。

## 境界

対象は package version、release/distribution、三言語 release/parity 記録、および
staged/public acceptance evidence である。参照プロジェクトの scaffold、Python/Make
実装、JSON wire format はコピーしない。object/adopter repository は read-only の外部
acceptance target とし、global Agent/MCP 設定と歴史的 governance bytes は対象外とする。

## Acceptance

1. Workspace package と `Cargo.lock` が `0.2.81` に解決される。
2. Release CI が annotated tag、target archive、SBOM/provenance、Formula、checksum、
   Runtime identity を一致する digest で公開する。
3. 公開後 adopter と N-1 acceptance は immutable な公開 artifact だけを使い、isolation
   と一時 run cleanup を証明する。
4. 英語・中国語・日本語の release/parity 記録が `v0.2.81` と N-1 `v0.2.80` の境界で一致する。
5. terminal Outcome は status、unknowns、evidence、decision、next action を含む、人向けの
   独立した可視 handoff とする。

## 検証

`cargo test --locked --workspace`、documentation/metadata、release policy/version
consistency、および immutable staged/public adopter harness を実行する。Release evidence
には download した Runtime の version と SHA-256 を記録し、terminal Outcome は独立した
人向け handoff として表示する。
