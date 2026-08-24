---
author: AI Cockpit maintainers
title: "WI-259 — close decision recovery と documentation projection"
workItemId: WI-259-close-decision-recovery
description: "immutable な lifecycle records を書き換えず predecessor の documentation projection を回復します。"
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-259-close-decision-recovery
authority: canonical
---

# WI-259 — close decision recovery と documentation projection

## Intent

WI-258 をそのまま保持し、close decision が満たせない documentation
projection を回復します。この successor は predecessor の実装、evidence、
human decision を解釈し直したり置き換えたりしません。

## Scope

変更範囲は三言語の WI-258 recovery projection、三言語の WI-259 record、
reference-parity rows、Runtime が生成する WI-258 recovery decision に限定します。
production Runtime、release artifacts、predecessor の `.ai` bytes は対象外です。

## Acceptance

- WI-258 archive、evidence、finalization、close bytes は byte-identical に保持する。
- recovery decision が正確な predecessor digests と successor ID を束縛する。
- 三言語の WI-258 docs/parity rows が Recovered になり、この successor を参照する。
- WI-259 自身の approved structured close と terminal evidence 後だけ Implemented に昇格する。
- documentation、parity、governance-integrity、promotion checks が通過する。

## Evidence boundary

Successor は audit projection と recovery boundary です。predecessor の説明的な
decision を `approved` と同一視せず、WI-259 の新しい明示的 close のみが自身の
terminal documentation promotion を許可します。
