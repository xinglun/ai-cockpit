---
title: "WI-610 — v0.2.82 release と adopter acceptance"
description: "次の Rust Runtime patch を公開し、immutable artifact と adopter 境界を検証する。"
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-610-release-v0-2-82
lastVerifiedBy: WI-610-release-v0-2-82
terminalArchive: .ai/work-items/archive/WI-610-release-v0-2-82.contract.json
terminalVerification: .ai/evidence/WI-610-release-v0-2-82.verification.json
terminalFinalization: .ai/decisions/WI-610-release-v0-2-82.finalize.json
terminalDecision: .ai/decisions/WI-610-release-v0-2-82.close.json
---

[English](WI-610-release-v0-2-82.md) · [简体中文](WI-610-release-v0-2-82.zh-CN.md)

# WI-610 — v0.2.82 release と adopter acceptance

## 目的

レビュー済み Rust Runtime を `v0.2.82` として公開し、immutable な公開 Release
artifact だけでインストールと upgrade を検証する。共有 Runtime/Repository 隔離モデルと、
直前の Work Item で検証した adopter cleanup の競合耐性を維持する。

## 境界

対象は package version metadata、release/distribution、三言語の release/parity 記録、
staged/public acceptance evidence である。対象リポジトリの変更、参照源の scaffold/Python/Make
実装のコピー、global Agent/MCP 設定の変更は行わない。

## 受入条件

1. Workspace package version と `Cargo.lock` が `0.2.82` に解決される。
2. Release CI が注釈付き `v0.2.82` tag、target archive、SBOM/provenance、Formula、checksum、
   一致した Runtime identity を公開する。
3. post-release adopter と N-1 acceptance は immutable な公開 artifact のみを使い、Repository
   隔離と一時実行ディレクトリの cleanup を証明する。
4. 英語・中国語・日本語の release/parity 記録が `v0.2.82` と `v0.2.81` N-1 境界を示す。
5. terminal Outcome は人間向けに表示され、status、unknowns、evidence、human decision、next
   action を記録する。

## 検証

`cargo test --locked --workspace`、docs/metadata、release policy/version consistency、immutable
staged/public adopter と N-1 harness を実行する。release evidence にダウンロードした Runtime
version と SHA-256 を記録する。terminal Outcome は独立した human handoff とする。
