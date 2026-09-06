---
title: "WI-625 — v0.2.84 release と adopter acceptance"
description: "direct-merge recovery 修正を公開し、immutable artifact を post-release adopter evidence で検証する。"
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-625-release-v0-2-84
lastVerifiedBy: WI-625-release-v0-2-84
---

[English](WI-625-release-v0-2-84.md) · [简体中文](WI-625-release-v0-2-84.zh-CN.md)

# WI-625 — v0.2.84 release と adopter acceptance

## 目的

レビュー済み Runtime を `v0.2.84` として公開する。この版は first-record
`direct_merge_no_pr` CLI round-trip 修正を含み、公開後に immutable artifact だけで
adopter と N-1 acceptance を実行する。

## 境界

対象は version metadata、release documentation、release policy、post-release evidence。
object repository、reference scaffold/Python/Make 実装、global Agent/MCP 設定は変更しない。

## 受入れ

1. Workspace package と `Cargo.lock` が `0.2.84` に解決される。
2. Release CI が注釈付き `v0.2.84` tag、target archive、checksum、SBOM/provenance、
   Formula、Runtime identity を公開する。
3. Public adopter/N-1 は immutable な `v0.2.84`/`v0.2.83` artifact のみを使い、
   repository isolation と run-root cleanup を証明する。
4. 英語・中国語・日本語の release/version/parity 記録が新 release と `v0.2.83` N-1
   boundary を示す。
5. 終端 Outcome は人間向けに表示し、status、unknowns、evidence、human decision、
   next action を記録する。

## シナリオカバレッジ

- `v0.2.84 source and artifact identity`: version、annotated tag、manifest、archive、SBOM、checksum、Runtime identity を一つのレビュー済み commit に束ねる。
- `public adopter and N-1 upgrade`: 不変の v0.2.84/v0.2.83 artifact だけを使い、repository isolation と temporary root cleanup を証明する。
- `direct_merge_no_pr recovery availability`: 生成 evidence を手編集せず、公開 Runtime の plan-to-apply 往復を検証する。

## 検証

workspace test、documentation/release policy/version check、公開 artifact の adopter/N-1
 harness を実行し、download した Runtime version と SHA-256 を release evidence に記録する。
source checkout や workspace binary は release evidence の代替にしない。
