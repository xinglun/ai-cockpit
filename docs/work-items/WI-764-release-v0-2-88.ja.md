---
author: AI Cockpit maintainers
title: WI-764 — v0.2.88 リリース
description: 次の Runtime release を公開し、不変な公開 artifact と adopter 境界を検証する。
audience: [adopter, maintainer, reviewer]
workItemId: WI-764-release-v0-2-88
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-764-release-v0-2-88
terminalArchive: .ai/work-items/archive/WI-764-release-v0-2-88.contract.json
terminalVerification: .ai/evidence/WI-764-release-v0-2-88.verification.json
terminalFinalization: .ai/decisions/WI-764-release-v0-2-88.finalize.json
terminalDecision: .ai/decisions/WI-764-release-v0-2-88.close.json
capabilityClaims: [release_distribution, reference_comparison, adopter_acceptance]
---

# WI-764 — v0.2.88 リリース

[English](WI-764-release-v0-2-88.md) · [简体中文](WI-764-release-v0-2-88.zh-CN.md)

## Intent

完了した performance measurement Work Item の後に v0.2.88 を公開し、不変な公開
Release artifact、checksum、SBOM/provenance、install boundary、隔離 adopter
acceptance、N-1 upgrade boundary を検証する。

## Boundary

この Work Item は Runtime package version と current release documentation を更新する。
Runtime behavior、reference source、過去の governance/release bytes、global Agent/MCP
configuration は変更しない。provider Release を先に作成せず、reserved tag も再利用しない。
Release acceptance は不変な公開 artifact だけを使い、source checkout や workspace binary
を代用しない。

## Verification

reviewed PR、source quality、version consistency、release policy、五つの target build、
manifest/checksum、SBOM/provenance、platform smoke、staged/public adopter、N-1 upgrade
check が成功しなければならない。download した binary は v0.2.88 を報告し、public manifest
と digest に一致し、repository が `ready_on_base` に戻ることを確認する。

close 後は front matter の terminal record が Contract、verification、finalization、decision
の authoritative evidence になる。
