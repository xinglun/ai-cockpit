---
author: AI Cockpit maintainers
title: "WI-398 — v0.2.40 documentation promotion"
description: "不変の Runtime 証拠に基づき、close 済み v0.2.40 release-preparation 文書を終端投影へ昇格する。"
workItemId: WI-398-release-v0-2-40-doc-promotion
audience: [maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-398-release-v0-2-40-doc-promotion
capabilityClaims: [documentation_governance, release_distribution]
---

# WI-398 — v0.2.40 documentation promotion

[English](WI-398-release-v0-2-40-doc-promotion.md) · [简体中文](WI-398-release-v0-2-40-doc-promotion.zh-CN.md)

## Intent

不変の archive、verification、finalization、close record を消費し、v0.2.40
release tag 作成前に close 済み WI-397 の文書を監査可能な終端投影へ昇格する。
これらの record 自体は書き換えない。

## Boundary

対象は三言語 WI-397 文書と parity ledger の status/終端リンク、および
レビュー済み delivery に必要な WI-397 close/finalization receipt の保持だけである。
Runtime 挙動、release 実装、公開 adopter acceptance は対象外とする。

## Verification

昇格 script、documentation acceptance、status consistency、governance integrity、
diff check を merge 前に通過させる。公開 binary と adopter acceptance は後続 Work Item が担当する。
