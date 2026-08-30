---
author: AI Cockpit maintainers
title: "WI-409 — v0.2.41 release と adopter acceptance"
description: "WI-408 後のレビュー済み Runtime を公開し、新しい adopter で immutable artifact を検証する。"
workItemId: WI-409-release-v0-2-41
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-409-release-v0-2-41
terminalArchive: .ai/work-items/archive/WI-409-release-v0-2-41.contract.json
terminalVerification: .ai/evidence/WI-409-release-v0-2-41.verification.json
terminalFinalization: .ai/decisions/WI-409-release-v0-2-41.finalize.json
terminalDecision: .ai/decisions/WI-409-release-v0-2-41.close.json
capabilityClaims: [release_distribution, adopter_acceptance, repository_isolation]
---

# WI-409 — v0.2.41 release と adopter acceptance

[English](WI-409-release-v0-2-41.md) · [简体中文](WI-409-release-v0-2-41.zh-CN.md)

## Intent

レビュー済みの WI-408 後の `main` から v0.2.41 を公開し、download した
immutable Release binary が reference source や V1 Runtime の残留物をコピー
せず、新しい adopter を attach・govern できることを証明する。

## Boundary

この Work Item は patch version、現在の三言語 release/versioning 文書、厳格な
release workflow、公開 adopter/N-1 acceptance を扱う。governance semantics、
historical evidence、global Agent/MCP 設定、無関係な adopter source は変更しない。
Runtime の distribution と repository attach は分離したままにする。

## Acceptance

1. Cargo metadata と lockfile を v0.2.40 から一つだけ patch 更新して v0.2.41
   とし、既存 tag/Release を再利用しない。
2. レビュー済み workflow が正確な merge commit から target を build し、manifest、
   Formula、SHA256SUMS、SBOM、provenance、immutable tag/Release identity を bind する。
3. 公開 v0.2.41 Release から download した binary を checksum 検証し、Runtime
   version、archive digest、binary digest、platform、download source を記録する。
   source/workspace fallback は使わない。
4. 新しい adopter acceptance が attach/profile/agent doctor、`first-adopter-smoke`
   の `not_ready`、lifecycle/evidence reuse、repository/Runtime isolation、temporary
   root cleanup を証明する。
5. 本 repository と新しい adopter が共有 Runtime 経由で WI-408 の read-only
   `work-item inspect` boundary を継承する。
6. review merge、finalization、close、同期、正確な branch cleanup、release 文書の
   promotion 後も `main` が ready on base である。

## Verification boundary

公開前は strict source/staged gate、公開後は immutable public artifact のみを使って
Runtime、checksum、adopter、isolation、cleanup evidence を保存する。公開済み Release
の失敗状態を後から書き換えない。ORG-X などの adopter は reference source の残留物を
コピーせず検査する。
