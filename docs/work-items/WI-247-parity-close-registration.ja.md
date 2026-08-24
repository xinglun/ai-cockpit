---
author: AI Cockpit maintainers
title: "WI-247 — WI-246 close parity registration"
workItemId: WI-247-parity-close-registration
description: "不変の WI-246 terminal decision chain を 3 言語の parity ledger に投影します。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-247-parity-close-registration
authority: canonical
---

# WI-247 — WI-246 close parity registration

PR #197 の merge、governance append の観測、正確な branch と feature worktree の削除後、
WI-246 は正しく close されました。authoritative close receipt の永続化により ledger の
順序 gap が明らかになりました。3 言語の parity row はまだ WI-246 を `In progress` とし、
canonical pre-merge receipt だけを列挙していたため、gate は期待どおり
`missing_parity_decision` と `stale_parity_status` を報告しました。

## Recovery boundary

Runtime が生成した recovery receipt は、正確な WI-246 Contract、Summary、Outcome、Events、
finalization chain、close identity を束縛します。これらの record、PR #197、merge commit
`98d6575` は不変です。WI-247 は英語、簡体字中国語、日本語の parity/Work Item projection
だけを変更し、Runtime、CI、release、tests、crates、WI-241 は変更しません。

## Acceptance and verification

各 WI-246 parity row を `Implemented` にし、canonical pre-merge receipt を保持したまま、
sequence-1 merge observation、sequence-2 cleanup、close、recovery path を追加します。
focused parity、governance、manifest、documentation check と canonical strict repository
runner はすべて pass しなければなりません。実在する draft PR #198 は verification 前に
束縛され、Runtime lifecycle record は documentation change と分離して保持されます。
