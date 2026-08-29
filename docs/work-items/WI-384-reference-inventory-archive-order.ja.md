---
author: AI Cockpit maintainers
title: "WI-384 — reference inventory archive order"
workItemId: WI-384-reference-inventory-archive-order
description: "origin/main から reference inventory parity 文書を再配信し、snapshot-bound evidence を守る finish/archive 順序を検証する。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-384-reference-inventory-archive-order
---

# WI-384 — reference inventory archive order

## Intent と境界

WI-384 は不変な WI-383 の明示的な recovery successor です。WI-383 では
`verify` と `archive` の間に生成 lifecycle records を commit したため、Runtime
が evidence を正しく拒否しました。この Work Item は WI-382 と WI-383 の全 bytes
を保持し、clean な `origin/main` から同じ bounded documentation correction を
再配信します。

## Scope と acceptance

三言語の comparison ページは 5,119 件の inventory marker と一致しなければ
なりません。三言語の parity ledger は verification 前に WI-382 と WI-383 を
Recovered、WI-384 を現在の delivery として登録します。WI-382、WI-383、WI-384
の Work Item ページは同一の identity/status metadata を持ちます。

順序も acceptance の一部です。reviewed PR を bind し、`verify`、`finish`、
`archive` の順に実行し、archive 成功後にだけ生成 lifecycle records を commit
します。predecessor bytes、Runtime、protocol、inventory 分類、CI/release logic、
global Agent/MCP configuration は変更しません。

## Verification

installed Runtime を明示的な repository path で使用し、inventory、documentation
status、governance-integrity checks を実行します。最終 Outcome は人に見える handoff
でなければなりません。green Outcome は merge や release の許可ではありません。

[English](WI-384-reference-inventory-archive-order.md) ·
[简体中文](WI-384-reference-inventory-archive-order.zh-CN.md)
