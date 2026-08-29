---
author: AI Cockpit maintainers
title: "WI-401 — v0.2.40 公開 Release 公開と adopter 受入れ"
description: "レビュー済み v0.2.40 Runtime を公開し、新規 adopter で不変 artifact を受入れる。"
workItemId: WI-401-release-v0-2-40-publication
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-401-release-v0-2-40-publication
terminalArchive: .ai/work-items/archive/WI-401-release-v0-2-40-publication.contract.json
terminalVerification: .ai/evidence/WI-401-release-v0-2-40-publication.verification.json
terminalFinalization: .ai/decisions/WI-401-release-v0-2-40-publication.finalize.json
terminalDecision: .ai/decisions/WI-401-release-v0-2-40-publication.close.json
capabilityClaims: [release_distribution, adopter_acceptance, runtime_installation]
---

# WI-401 — v0.2.40 公開 Release 公開と adopter 受入れ

[English](WI-401-release-v0-2-40-publication.md) · [简体中文](WI-401-release-v0-2-40-publication.zh-CN.md)

## Intent

レビュー済みで同期された `main` から v0.2.40 を公開し、不変の公開 artifact が
新規 adopter と本リポジトリを治理できることを確認する。

## Boundary

対象は tag/Release 公開、不変 artifact の受入れ、検証済み binary のインストール、
監査可能な外部受入れ証拠だけである。Runtime semantics、reference parity、business
project code、global Agent/MCP configuration は変更しない。公開受入れは source build
や workspace binary への fallback を拒否する。

## Acceptance

1. v0.2.40 tag と公開 Release はレビュー済み main merge から作成され、Release identity、
   SBOM、provenance、checksum gate を通過する。
2. ダウンロードした artifact に tag、version、archive digest、binary digest、platform、
   download source を記録し、その identity を evidence に bind する。
3. 新規 adopter は独立した repository identity と完全な lifecycle を持ち、
   `first-adopter-smoke` は `not_ready` のままで、evidence reuse と forbidden roots・
   temporary run root の cleanup を証明する。
4. 検証済み公開 binary を本リポジトリへインストールし `COMPATIBLE` と `doctor=ok` を確認し、
   main は同期済みで ready on base になる。

## Verification boundary

Post-release acceptance は公開 artifact の証拠であり、Release truth や履歴 evidence を
書き換えない。失敗時は `releasePublished: true` と `adopterAcceptance: failed` を記録する。
