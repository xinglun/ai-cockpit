---
title: "WI-616 — v0.2.83 release と adopter acceptance"
description: "direct-merge recovery 修正を公開し、immutable な Release artifact を adopter/N-1 acceptance で検証する。"
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-616-release-v0-2-83
lastVerifiedBy: WI-616-release-v0-2-83
---

[English](WI-616-release-v0-2-83.md) · [简体中文](WI-616-release-v0-2-83.zh-CN.md)

# WI-616 — v0.2.83 release と adopter acceptance

## 目的

レビュー済み Rust Runtime を `v0.2.83` として公開し、immutable な公開 Release artifact だけで install と upgrade を検証する。この Release は adopter repository 向けの WI-614 first-record `direct_merge_no_pr` recovery 修正を含む。

## 境界

この Work Item は package version metadata、release/distribution、三言語の release/parity 記録、staged/public acceptance evidence を対象とする。adopter repository、reference scaffold/Python/Make、グローバル Agent/MCP 設定は変更しない。

## 受入れ

1. Workspace package version と `Cargo.lock` が `0.2.83` に解決される。
2. Release CI が注釈付き `v0.2.83` tag、target archive、SBOM/provenance、Formula、checksum、Runtime identity を公開する。
3. Public adopter/N-1 acceptance は immutable な `v0.2.83`/`v0.2.82` artifact だけを使い、repository isolation と一時 run root cleanup を証明する。
4. 英語・中国語・日本語の release/versioning/parity 記録が `v0.2.83` と `v0.2.82` N-1 境界を示す。
5. 終端 Outcome は人間向けに表示し、status、unknowns、evidence、human decision、next action を記録する。

## 検証

workspace test、documentation/metadata、release policy/version consistency、immutable staged/public adopter と N-1 harness を実行し、download した Runtime version と SHA-256 を release evidence に記録する。
