---
author: AI Cockpit maintainers
title: "WI-855 — 選択済み successor lineage の復旧"
description: "歴史 evidence を書き換えたり競合する successor を作成したりせず、選択済みの複数段 successor lineage を復旧する。"
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
workItemId: WI-855-selected-successor-lineage-recovery
lastVerifiedBy: WI-855-selected-successor-lineage-recovery
terminalArchive: .ai/work-items/archive/WI-855-selected-successor-lineage-recovery.contract.json
terminalVerification: .ai/evidence/WI-855-selected-successor-lineage-recovery.verification.json
terminalDecision: .ai/decisions/WI-855-selected-successor-lineage-recovery.close.json
---

[English](WI-855-selected-successor-lineage-recovery.md) · [简体中文](WI-855-selected-successor-lineage-recovery.zh-CN.md)

# WI-855 — 選択済み successor lineage の復旧

WI-855 は、すでに選択された複数段 successor lineage のための append-only
recovery receipt を追加します。隣接する各 edge と各 archived node を、正確な
repository bytes、Runtime identity、provider PR/finalization facts、明示的な
human decision に bind します。invalid、stale、foreign、fork、ambiguous な履歴は
fail-closed のままで、競合 successor の作成や歴史 record の書き換えは行いません。
