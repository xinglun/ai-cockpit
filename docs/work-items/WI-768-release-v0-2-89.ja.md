---
author: AI Cockpit maintainers
title: WI-768 — v0.2.89 release recovery
description: 厳格な release quality 境界を修復し、v0.2.88 の失敗履歴を保持したまま新しい不変 Release を公開する。
audience: [adopter, maintainer, reviewer]
workItemId: WI-768-release-v0-2-89
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-768-release-v0-2-89
capabilityClaims: [release_distribution, adopter_acceptance, governance_evidence]
---

[English](WI-768-release-v0-2-89.md) · [简体中文](WI-768-release-v0-2-89.zh-CN.md)

# WI-768 — v0.2.89 release recovery

## Intent

v0.2.88 の immutable な失敗履歴を書き換えずに release path を復旧し、active Contract-aware
Rust gate と repository gate receipt を bind する release workflow で v0.2.89 を公開する。

## Boundary

この Work Item は release workflow の quality binding、Runtime package version、current
release/version documentation、公開 artifact と adopter boundary の検証 evidence だけを変更する。
Runtime の production behavior、performance implementation、WI-764/v0.2.88 の historical bytes、
global Agent/MCP configuration、tag の再利用、provider Release の先行作成は対象外である。

## Verification

reviewed PR の hosted checks と v0.2.89 release workflow は、Contract-aware source-quality receipt、
repository gate receipt、五つの target archive、SBOM/provenance、manifest/checksum、release identity
を証明しなければならない。public adopter installation と v0.2.87 から v0.2.89 への upgrade acceptance
は download した immutable artifact だけを使い、isolation/cleanup evidence を保持する。

Runtime lifecycle 完了後に、terminal Contract、verification、release、finalization、cleanup、decision
path をこのページに追加する。
