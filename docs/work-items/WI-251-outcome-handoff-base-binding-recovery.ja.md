---
author: AI Cockpit maintainers
title: "WI-251 — Outcome handoff base binding recovery"
workItemId: WI-251-outcome-handoff-base-binding-recovery
description: "Lifecycle Outcome を再 delivery し、archived Contract/PR base mismatch を resource finalization で拒否します。"
audience:
  - adopter
  - maintainer
status: current
lastVerifiedBy: WI-251-outcome-handoff-base-binding-recovery
authority: canonical
---

# WI-251 — Outcome handoff base binding recovery

WI-250 は不変の verified archive と canonical finalization receipt を生成しましたが、
rebase 後に archived Contract base と provider PR base が異なることを hosted governance
が検出しました。インストール済み Runtime はその sequence-0 receipt を verified と
報告していました。WI-251 は predecessor bytes を保持し、recovery decision を bind して、
正しい current base から Outcome handoff を再 delivery します。

## Behavior

- 直接 lifecycle handoff は後方互換です。`finish`、`archive`、`close` は stdout JSON
  を維持し、既定で検証済み Human Outcome を stderr に render し、`--json` はその
  handoff を抑止します。
- block された `finish` は永続化済みの赤/黄 Outcome を表示し、元の nonzero result
  を維持します。
- Resource finalization record は `pullRequest.baseRevision` と archived Contract の
  `baseRevision` が異なる receipt を canonical/transition decision の書き込み前に拒否します。
- `finalize-verify` も canonical sequence 0 を含め同じ cross-binding を検査し、保存済み
  mismatch を verified と報告しません。

## Immutable boundary

archive は Contract base を凍結します。rebase は Work Item が active の間に行い、Contract
binding と review を更新します。archive 後の base 変更は fail closed recovery とし、archive
も finalization receipt も書き換えません。WI-250 の archive、evidence、Outcome、events、
finalization bytes は historical truth のまま保持され、recovery decision が WI-251 を指します。

## Verification

repository regression は decision file を作らない record rejection、制御された fixture
tamper 後の sequence-0 verify rejection、matching-base success、既存 transition control を
検証します。CLI test は 3 言語、stdout compatibility、`--json`、structured decision、blocked
handoff を検証します。documentation、parity、governance、format、Clippy、locked workspace
suite も必須です。
