---
author: AI Cockpit maintainers
title: "WI-446 — 歴史 finalization recovery"
workItemId: WI-446-historical-finalization-recovery
description: "legacy finalization record を正直に append-only recovery する path。"
audience: [maintainer, adopter, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-446-historical-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-446-historical-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-446-historical-finalization-recovery.verification.json
terminalFinalization: .ai/decisions/WI-446-historical-finalization-recovery.finalize.json
terminalDecision: .ai/decisions/WI-446-historical-finalization-recovery.close.json
---

# WI-446：歴史 finalization recovery

## Intent

専用 linked worktree とレビュー済み PR の workflow より前に作られた legacy finalization record のため、正直な append-only 互換 path を提供します。Runtime は predecessor receipt を書き換えず、PR 番号も捏造せずに adopter の歴史的な handoff を完了できなければなりません。

## Scope

- 旧 shared-primary-worktree の `retained` receipt を Runtime-bound な `historical_finalization_recovery` record で分類する；
- repository、Work Item、Contract base、predecessor digest、Runtime、human authority の binding を検証する；
- 実際の merge commit、parents、base、Git facts が一致する場合だけ no-PR direct-merge receipt を受け入れる；
- 明示的な low-assurance の歴史 close を許可し、新しい Work Item の deleted-resource gate は維持する；
- `work-item finalize-recovery` command と三言語 documentation を提供する。

## Non-goals

歴史 bytes の書き換え、PR 番号の捏造、current Runtime identity check の緩和、object repository の自動 migration は行いません。歴史 assurance は `historical_low` のままで、provider assurance には昇格しません。

## Acceptance

repository tests は shared-worktree recovery、foreign/tampered/symlink の拒否、実 Git parents による direct merge 検証、predecessor を書き換えない close をカバーします。command/workflow reference は互換境界と human-authorized recovery command を説明します。
