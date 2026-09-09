---
author: AI Cockpit maintainers
title: "WI-746 — WI-745 parity 順序リカバリ successor"
description: "parity 登録を先に commit し、新しい verification evidence を後から記録する限定的なドキュメント復旧を再提供する。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-746-wi745-recovery-successor
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-746-wi745-recovery-successor
---

[English](WI-746-wi745-recovery-successor.md) · [简体中文](WI-746-wi745-recovery-successor.zh-CN.md)

# WI-746 — WI-745 parity 順序リカバリ successor

## Intent

WI-745 を immutable predecessor として保持し、「parity 登録を先に、検証
証拠を後に」する境界で限定的な documentation-governance recovery を再提供
します。この Work Item は WI-743 と WI-745 の historical evidence を変更
しません。

## Boundary

対象は三言語の WI-746 ドキュメント投影と、三つの reference-parity 台帳行
だけです。production code、Runtime の動作、governance rule、WI-743 または
WI-745 の記録、PR #718、他 agent の Work Item は変更しません。

## Evidence and lifecycle

- WI-745 recovery decision が predecessor の明示的な binding です。
- ドキュメントページと parity 行を先に commit します。
- 新しい Runtime verification evidence はその commit の後だけ記録します。
- lifecycle は `start → preflight → checkpoint → verify → finish → archive
  → close` で、finalization までは terminal path を planned とします。

## Acceptance

documentation acceptance、parity status check、Work Item status consistency
check が pass し、Git history で「登録が証拠に先行する」順序を確認できる
こと。historical record や production behavior は書き換えません。
