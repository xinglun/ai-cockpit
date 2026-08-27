---
author: AI Cockpit maintainers
title: "WI-340 — archived finalization recovery"
workItemId: WI-340-finalization-recovery
description: "provider finalization が保留中の archived Work Item に、限定的で append-only な recovery path を提供します。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-340-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-340-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-340-finalization-recovery.verification.json
terminalFinalization: .ai/decisions/WI-340-finalization-recovery.finalize.db551e5edf1e88fde01c18898a6a81b58692f425d427d71aeee3442b4e90d613.json
terminalDecision: .ai/decisions/WI-340-finalization-recovery.close.json
---

# WI-340 — archived finalization recovery

WI-340 は provider context を持ちながら provider-side finalization receipt が
まだ存在しない archived Work Item の recovery 境界を明確にします。通常の
archived Work Item は receipt が記録されるまで non-green のままです。履歴上の
predecessor に対する限定的な例外は、有効な append-only `supersede` recovery
decision によってのみ許可されます。

元の Contract、Summary、Outcome、Events、archive、verification evidence は
immutable のまま保持します。欠落、無効、foreign、malformed な recovery record
で finalization や evidence の gate を迂回することはできません。正常に
finalization 済みの Work Item は既存の green path を維持します。

文書入口：[English](WI-340-finalization-recovery.md) · [简体中文](WI-340-finalization-recovery.zh-CN.md)

## Acceptance boundary

1. 有効な supersede recovery decision は predecessor の archive bytes を書き換えずに明示的な close flow を許可します。
2. 欠落または無効な recovery decision は finalization / verification gate を迂回できません。
3. provider finalization pending は人間向け Outcome で yellow として表示し、verified / green にはしません。
4. 正常に finalization された Work Item は既存の green path を維持します。
5. recovery、pending-finalization、invalid decision、finalized path の locked workspace regression が通過します。
