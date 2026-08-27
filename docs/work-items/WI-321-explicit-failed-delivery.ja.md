---
author: AI Cockpit maintainers
title: "WI-321 — 明示的 failed-delivery recovery boundary"
workItemId: WI-321-explicit-failed-delivery
description: "immutable な failed-delivery history を書き換えず、Runtime-bound successor を記録する。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-321-explicit-failed-delivery
---

# WI-321 — 明示的 failed-delivery recovery boundary

## Intent と boundary

WI-313 は immutable な failed delivery です。PR #277 は merge されず、その retry
receipt に terminal decision または successor がありません。本 Work Item は Runtime が
生成した successor receipt を記録し、governance gate が history を orphaned のまま残したり、
完了した implementation として暗黙に投影したりしないようにします。

predecessor は historical truth として保持します。本 Work Item は Contract、Summary、
Outcome、Events、archive、verification、retry receipt、branch、PR の記録を改変・削除せず、
WI-313 の implementation が merge 済みだとも主張しません。

## Scope と acceptance

- Runtime-generated WI-313 successor receipt は本 Work Item、repository identity、
  predecessor digest、Runtime identity、明示的 human authority に bind されます。
- governance integrity gate は、successor のない orphaned retry を terminal success として
  扱えず、明示的 successor は `Recovered` として受理する deterministic regression を持ちます。
- English、Simplified Chinese、日本語の Work Item/parity projection は未 merge の PR 失敗境界を
  記述し、recovery receipt を evidence として参照します。
- 既存の historical bytes と global Agent/MCP configuration は変更しません。

## Verification

orphaned-retry/recovery-chain static regression、documentation acceptance、locked workspace
test、review 済み branch の hosted CI を実行します。repository-bound Runtime command は常に
explicit repository path を使い、source-build fallback は release evidence にしません。

## Related history

- WI-313: immutable な PR #277 failed delivery。本 successor が明示的に recovery します。
- WI-314、WI-315: 独立した recovery chain として変更しません。

[English](WI-321-explicit-failed-delivery.md) ·
[简体中文](WI-321-explicit-failed-delivery.zh-CN.md)
