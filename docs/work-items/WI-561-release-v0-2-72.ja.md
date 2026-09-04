---
author: AI Cockpit maintainers
title: "WI-561 — v0.2.72 release と公開 artifact acceptance"
description: "次の immutable AI Cockpit Runtime Release を公開し検証する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-561-release-v0-2-72
lastVerifiedBy: WI-561-release-v0-2-72
---

[English](WI-561-release-v0-2-72.md) · [简体中文](WI-561-release-v0-2-72.zh-CN.md)

# WI-561 — v0.2.72 release と公開 artifact acceptance

## Objective

review 済み default branch から v0.2.72 を immutable Runtime baseline として公開し、
source checkout や workspace fallback を使わず、download した public binary がこの
repository を governance できることを確認する。

## Scope と boundary

- English、簡体中文、日本語の Cargo metadata、lockfile、current release/versioning guidance を同期する。
- default branch で close 済みの reference comparison と documentation promotion record に Release を bind する。
- 五つの target archive、manifest、checksum、SBOM、provenance、attestation、Runtime identity を生成・検証する。
- immutable に download した artifact だけで isolated root の public adopter と N-1 acceptance を実行し、forbidden root と temporary run root の cleanup を証明する。

Object repository、global Agent/MCP configuration、Runtime behavior、source-template copy、failed tag の書き換え、無関係な historical record はこの WI の範囲外とする。

## Acceptance

1. Cargo metadata、lockfile、current release/versioning page が v0.2.72 を示し、v0.2.71 を直前の public baseline として保持する。
2. Release CI が identity-bound な五 target artifact と supply-chain receipt 一式を生成する。
3. Public adopter と N-1 acceptance は v0.2.72 の download artifact だけを使い、isolation と cleanup を証明し、同じ binary でこの repository を検証する。
4. Release は synchronized で ready な default branch から開始し、Runtime behavior、object repository、global configuration、無関係な historical evidence を変更しない。

## Verification boundary

Contract の acceptance は作成言語の原文を authority とし、localized page は presentation
だけを変更する。immutable な public asset と adopter receipt の検証が終わるまで Release
を accepted とみなさない。object repository の acceptance は外部 read-only handoff であり、
この page はそれを代行して主張しない。
