---
author: AI Cockpit maintainers
title: "WI-500 — historical archive integrity recovery"
description: "任意レポート bytes が manifest digest と一致しない immutable historical archive のための監査可能な限定 recovery path を追加します。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-500-historical-archive-recovery-integrity
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-500-historical-archive-recovery-integrity
canonical: docs/work-items/WI-500-historical-archive-recovery-integrity.ja.md
---

# WI-500 — historical archive integrity recovery

[English](WI-500-historical-archive-recovery-integrity.md) · [简体中文](WI-500-historical-archive-recovery-integrity.zh-CN.md)

## Boundary

この Work Item は、任意の `taskReportMarkdown` bytes が記録済み manifest
digest と異なる immutable historical archive に対して、限定的で fail-closed
な recovery path を追加します。必須の identity、Contract、Summary、Outcome、
その他 artifact の binding は厳格に維持し、predecessor bytes は書き換えません。

## Delivery state

実装は専用 branch で archive と verification を完了しています。review 済み PR
の merge と正確な resource cleanup を記録するまで、provider finalization と close
は保留します。
