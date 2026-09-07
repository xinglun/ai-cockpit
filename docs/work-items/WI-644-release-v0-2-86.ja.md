---
author: AI Cockpit maintainers
title: WI-644 — v0.2.86 リリース
description: 参考ソースの全ファイル比較完了後に Runtime を公開し、公開 artifact の境界を検証する。
audience: [adopter, maintainer, reviewer]
workItemId: WI-644-release-v0-2-86
status: implemented
authority: human-authorized
lastVerifiedBy: WI-644-release-v0-2-86
terminalArchive: .ai/work-items/archive/WI-644-release-v0-2-86.contract.json
terminalVerification: .ai/evidence/WI-644-release-v0-2-86.verification.json
terminalFinalization: .ai/decisions/WI-644-release-v0-2-86.finalize.json
terminalDecision: .ai/decisions/WI-644-release-v0-2-86.close.json
capabilityClaims: [release_distribution, reference_comparison, adopter_acceptance]
---

# WI-644 — v0.2.86 リリース

[English](WI-644-release-v0-2-86.md) · [简体中文](WI-644-release-v0-2-86.zh-CN.md)

## Intent

6 回の reference-comparison batch 完了後に Runtime を公開します。レビュー済み commit を version metadata、checksum、SBOM/provenance、install 手順、downloaded-artifact adopter acceptance に binding します。

## Boundary

この Work Item は release/version projection のみを更新し、source implementation や source wire format を copy しません。公開および N-1 adopter evidence は、release workflow が immutable published artifact から生成します。object repository は shared Runtime の独立した利用者です。

## Verification

PR review 前に workspace test、release policy、source archive、Action runtime、adopter wrapper、5,175 件の reference inventory、三言語 documentation、governance integrity check がすべて pass しました。公開 tag と post-release acceptance は release workflow の provider-bound evidence です。

権威ある Contract、verification、finalization、close evidence は front matter の terminal record を参照してください。
