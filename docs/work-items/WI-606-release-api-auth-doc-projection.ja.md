---
title: "WI-606 — WI-605 終端ドキュメント投影"
description: "ホスト型ドキュメントゲートが要求する三言語 parity 記録を昇格する。"
author: AI Cockpit maintainers
audience: [maintainer, reviewer]
status: in_progress
authority: canonical
workItemId: WI-606-release-api-auth-doc-projection
predecessorWorkItemId: WI-605-release-api-auth
---

[English](WI-606-release-api-auth-doc-projection.md) · [简体中文](WI-606-release-api-auth-doc-projection.zh-CN.md)

# WI-606 — WI-605 終端ドキュメント投影

## 目的

三言語の reference-parity index と Work Item 文書を、公開リリース受入れ修正と整合させる。
これは限定されたドキュメント successor であり、Runtime の動作や object repository を変更しない。

## 境界

対象は三言語 parity index と WI-605 の三言語文書である。前任 Work Item の archive、evidence、recovery バイトは不変のままにする。Runtime、リリースハーネス、インストーラーの動作は対象外である。

## 検証

`bash tests/docs/parity_status_check.sh .` と Contract に宣言した workspace 検証を実行する。close 前にレビュー済み PR と hosted checks の成功を必要とする。
