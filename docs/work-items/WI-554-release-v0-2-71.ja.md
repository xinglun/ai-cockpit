---
author: AI Cockpit maintainers
title: "WI-554 — v0.2.71 release と公開 artifact acceptance"
description: "Capability surface と文書修正を immutable Runtime release として公開します。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-554-release-v0-2-71
lastVerifiedBy: WI-554-release-v0-2-71
terminalArchive: .ai/work-items/archive/WI-554-release-v0-2-71.contract.json
terminalVerification: .ai/evidence/WI-554-release-v0-2-71.verification.json
terminalFinalization: .ai/decisions/WI-554-release-v0-2-71.finalize.json
terminalDecision: .ai/decisions/WI-554-release-v0-2-71.close.json
---

[English](WI-554-release-v0-2-71.md) · [简体中文](WI-554-release-v0-2-71.zh-CN.md)

# WI-554 — v0.2.71 release と公開 artifact acceptance

## Objective

review 済み default branch から v0.2.71 を次の immutable Runtime baseline として公開します。Capability registry、capability discovery 文書、WI-552 reference comparison を含み、公開済み v0.2.70 は歴史的 N-1 evidence として保持します。

## Scope と boundary

- Cargo metadata/lockfile と英語・中国語・日本語の current release、distribution、versioning guidance を v0.2.71 に揃えます。
- close 済みで promotion 済みの WI-552 と WI-553 に release を bind します。
- 五つの target artifact、manifest、checksum、SBOM、provenance、attestation、post-release adopter receipt を生成・検証します。
- Object repository、global Agent/MCP config、source template copying、Runtime behavior change は範囲外です。

## Acceptance

1. Cargo metadata、lockfile、current release/versioning docs が v0.2.71 を示し、v0.2.70 は直前の公開 baseline、失敗 tag は immutable history のままです。
2. Release CI が identity-bound artifact set と supply-chain receipt を生成します。
3. Public adopter と N-1 acceptance は隔離 root の v0.2.71 download artifact だけを使い、cleanup/forbidden-root isolation と同じ binary による本 repository governance を検証します。
4. WI-552 と WI-553 は close・documentation promotion 済みで、clean かつ ready な default branch から release を開始します。

## Verification boundary

Contract prose は原言語を保持し、localization は表示だけを変更します。Object-repository acceptance は外部 read-only handoff であり、チームの receipt が届くまで主張しません。
