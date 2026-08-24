---
author: AI Cockpit maintainers
title: "WI-247 — WI-246 close parity registration"
workItemId: WI-247-parity-close-registration
description: "WI-247 の immutable archive を保持し、archive 後の parity 順序 defect を recovery します。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-249-parity-finalization-registration
authority: canonical
---

# WI-247 — WI-246 close parity registration

WI-247 は authoritative な WI-246 close chain を投影する documentation change を verify し、
archive しました。archive 後に自身の parity row を active Contract projection から
archive/evidence/finalization path へ変更しました。この documentation mutation は archived
verification snapshot に含まれないため、PR #198 は green delivery ではなく unmerged で
immutable な predecessor として保持されます。

## Recovery boundary

Runtime receipt `.ai/decisions/WI-247-parity-close-registration.recovery.json` は、正確な
Contract、Summary、Outcome、Events、archive manifest、verification evidence、repository
identity、Runtime v0.2.31 digest を束縛します。WI-249 は recovery bootstrap `f59ff36`
からその bytes を変更せずに取り込み、WI-247 implementation の replay や finalization
receipt の捏造を行いません。

## Root correction

WI-249 は WI-247 を Recovered として保持し、WI-246 terminal ledger projection を完了し、
条件付き pre-archive control を追加します。Contract、Summary、acceptance が parity ledger
を所有する Work Item だけが verification 前に 3 つの lifecycle-bound row を公開します。
通常の code Work Item はこの documentation obligation の対象外です。
