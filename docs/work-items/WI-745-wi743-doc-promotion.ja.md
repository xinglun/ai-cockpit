---
author: AI Cockpit maintainers
title: "WI-745 — WI-743 終端ドキュメント昇格"
description: "検証済み WI-743 の終端ドキュメントを昇格し、hosted documentation-governance 投影を修復する。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-745-wi743-doc-promotion
status: recovered
authority: human:repository-owner
lastVerifiedBy: WI-745-wi743-doc-promotion
recoveryDecision: .ai/decisions/WI-745-wi743-doc-promotion.recovery.6ef6b1b0e804eaf246518a1839851b47b0de88cb9a18c73282f21fb0816d1e41.json
closeDecision: .ai/decisions/WI-745-wi743-doc-promotion.close.json
---

[English](WI-745-wi743-doc-promotion.md) · [简体中文](WI-745-wi743-doc-promotion.zh-CN.md)

# WI-745 — WI-743 終端ドキュメント昇格

## Intent

検証済み WI-743 の終端ドキュメントを昇格し、このドキュメント専用
successor を三言語の reference-parity 台帳へ登録します。hosted
documentation-governance の投影を修復するもので、closed 状態の WI-743
記録や merge 済みの performance delivery は変更しません。

## Boundary

この Work Item は WI-743 の六つのドキュメント投影と、archive 前に
hosted gate が要求する WI-745 の三言語 Work Item/parity 投影だけを対象
にします。production code、test、governance rule、WI-743 の archive・
decision、PR #716、WI-744、WI-742 の history は変更しません。

## Evidence and lifecycle

- 昇格内容の immutable source は WI-743 の terminal archive、verification、
  finalization、close records です。
- PR #718 は同期済み `origin/main` を基線とする documentation-only successor
  です。
- lifecycle は `start → preflight → checkpoint → verify → finish →
  archive → close` で、WI-745 は Runtime `supersede` により historical predecessor として
  close 済みです。後続の所有は WI-746 であり、本ページは新しい performance や
  user-visible benefit を主張しません。
- 現在の supersede receipt は `.ai/decisions/WI-745-wi743-doc-promotion.recovery.6ef6b1b0e804eaf246518a1839851b47b0de88cb9a18c73282f21fb0816d1e41.json`、
  close decision は `.ai/decisions/WI-745-wi743-doc-promotion.close.json` です。

## Acceptance

promotion helper、documentation acceptance、parity status check、Work Item
status consistency check が pass し、historical evidence を書き換えず、
production behavior や governance decision を追加しないことを確認します。
