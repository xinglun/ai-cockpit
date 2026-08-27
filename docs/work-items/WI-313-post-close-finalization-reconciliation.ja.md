---
author: AI Cockpit maintainers
title: "WI-313 — post-close finalization reconciliation"
workItemId: WI-313-post-close-finalization-reconciliation
description: "cleanup-before-close を強制し、immutable な legacy close record のための限定的な recovery path を追加する。"
audience:
  - maintainer
  - reviewer
status: recovered
authority: canonical
lastVerifiedBy: WI-313-post-close-finalization-reconciliation
---

# WI-313 — post-close finalization reconciliation（recovered history）

## Intent and boundary

W312 は、旧 Runtime が provider finalization が `retained` のまま Work Item を
close でき、closed-document promotion gate が cleanup 完了の主張を拒否する順序欠陥を
示しました。WI-313 は bounded correction を試みましたが、PR #277 は merge 前に hosted
quality によって正しく拒否されました。本ドキュメントは immutable な failed-delivery
history を記録するもので、merged implementation を主張しません。WI-321 が明示的な
successor-owned recovery を記録し、WI-313 の bytes は書き換えません。新しい Work Item は
close 前に provider resource を cleanup し、immutable な legacy close だけがその後に bound
deleted transition を 1 件追加できます。

## Scope and acceptance

元の Rust protocol/repository lifecycle correction と hosted delivery attempt は historical
evidence として保持します。現在の gate は、Runtime が生成し WI-321 に bind された successor
receipt によってのみ本 Work Item を `Recovered` と投影します。元の Contract、Summary、
Outcome、Events、archive、verification、retry receipt、branch、PR bytes は不変です。
documentation promotion gate と三言語 workflow は orphaned retry を拒否し、明示的 successor
または有効な terminal path を要求します。

## Verification

元の Rust finalization targeted tests と hosted PR evidence は historical です。WI-321 は
orphaned-retry static regression を追加し、三言語 recovery projection、documentation gates、
installed Runtime が生成した successor receipt を検証します。release acceptance の
source-build fallback は認めません。

[WI-321 successor recovery](WI-321-explicit-failed-delivery.ja.md)

[English](WI-313-post-close-finalization-reconciliation.md) ·
[简体中文](WI-313-post-close-finalization-reconciliation.zh-CN.md)
