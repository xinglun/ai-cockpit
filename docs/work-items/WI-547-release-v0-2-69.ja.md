---
author: AI Cockpit maintainers
title: "WI-547 — v0.2.69 release と公開 artifact acceptance"
description: "失敗した v0.2.68 の投影を修正し、新しい immutable Runtime baseline を公開する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-547-release-v0-2-69
lastVerifiedBy: WI-547-release-v0-2-69
terminalArchive: .ai/work-items/archive/WI-547-release-v0-2-69.contract.json
terminalVerification: .ai/evidence/WI-547-release-v0-2-69.verification.json
terminalFinalization: .ai/decisions/WI-547-release-v0-2-69.finalize.json
terminalDecision: .ai/decisions/WI-547-release-v0-2-69.close.json
---

[English](WI-547-release-v0-2-69.md) · [简体中文](WI-547-release-v0-2-69.zh-CN.md)

# WI-547 — v0.2.69 release と公開 artifact acceptance

## 目的

レビュー済みで同期された default branch から、事実に基づく v0.2.69 Runtime
baseline を公開する。失敗した v0.2.68 tag は immutable history として保持し、公開・installable version として扱わない。

## 範囲と境界

- package identity と lockfile。
- 三言語の release、versioning、distribution 文書。
- この release の Work Item と reference-parity projection。
- 公開 artifact、checksum、SBOM、adopter、installation acceptance は、この Work Item に束縛された release evidence とする。
- Runtime behavior、対象 repository、global Agent/MCP 設定、失敗した v0.2.68 tag は範囲外。

## Acceptance

1. package と文書が v0.2.69 を一貫して示し、v0.2.68 を失敗履歴として明記する。
2. Release CI と policy gate が immutable tag とレビュー済み source commit に束縛された manifest、SHA256SUMS、SBOM、provenance、公開 artifact を生成する。
3. download-only の公開 binary を隔離 root で受入れ、cleanup と forbidden-write isolation を証明し、同じ binary を install して明示的 repository health check を行う。

## 検証境界

Contract の acceptance prose は元の言語で保持し、見出しの localization は governance fact を翻訳しない。release evidence は tag、archive digest、binary digest、Runtime identity、adopter receipt を束縛する。失敗した公開は失敗として記録し、再利用しない。
