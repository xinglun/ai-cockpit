---
author: AI Cockpit maintainers
title: WI-646 — v0.2.87 リリース
description: Runtime を公開し、不変な公開 artifact と adopter 境界を検証する。
audience: [adopter, maintainer, reviewer]
workItemId: WI-646-release-v0-2-87
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-646-release-v0-2-87
capabilityClaims: [release_distribution, reference_comparison, adopter_acceptance]
---

# WI-646 — v0.2.87 リリース

[English](WI-646-release-v0-2-87.md) · [简体中文](WI-646-release-v0-2-87.zh-CN.md)

## Intent

Runtime の version binding 修正後に v0.2.87 を公開し、公開 Release artifact、
checksum、SBOM/provenance、install path、隔離 adopter acceptance を検証する。

## Boundary

この Work Item は Runtime version と release/documentation projection を更新する。
reference source のコピー、過去の governance bytes の書き換え、adopter repository
の操作、global Agent/MCP configuration の変更は行わない。Release acceptance は
不変な公開 artifact だけを使い、source workspace や local target binary を代用しない。

## Verification

workspace、documentation、release policy、source quality、public adopter、N-1
upgrade の各 check が成功しなければならない。公開 binary を install した後に
v0.2.87 を報告し、download digest が public manifest と receipt に一致し、repository
が `ready_on_base` に戻ることを確認する。

close 後は front matter の terminal record が Contract、verification、finalization、
decision の authoritative evidence になる。
