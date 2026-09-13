---
author: AI Cockpit maintainers
title: "WI-826 — 過去 Runtime の evidence archive 互換性"
description: "古い Runtime が取得した typed evidence を履歴バイトを書き換えずに明示的かつ fail-closed に archive する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized
workItemId: WI-826-historical-lifecycle-reconciliation
lastVerifiedBy: WI-827-wi826-docs-promotion
---

[English](WI-826-historical-lifecycle-reconciliation.md) · [简体中文](WI-826-historical-lifecycle-reconciliation.zh-CN.md)

# WI-826 — 過去 Runtime の evidence archive 互換性

## Intent と boundary

この Work Item は、過去の Runtime が取得した typed verification evidence
のために明示的な historical archive route を追加する。元の evidence bytes
を保持し、archive manifest を raw digest、Work Item、repository、Contract、
snapshot、Runtime、receipt の identity に bind する。通常の current Runtime
archive の動作は変えない。

## Fail-closed behavior

historical archive は current Runtime の evidence、malformed または legacy
untyped evidence、digest の改ざん、identity の不一致を拒否し、partial な
archive を書き込まない。validator は bind された evidence file と全 identity
を再検査する。この compatibility route は明示的に呼び出す必要があり、
一般的な historical exemption にはならない。

## Acceptance evidence

- Archive: `.ai/work-items/archive/WI-826-historical-lifecycle-reconciliation.contract.json`
- Formal verification: `.ai/evidence/WI-826-historical-lifecycle-reconciliation.verification.json`
- Close decision: `.ai/decisions/WI-826-historical-lifecycle-reconciliation.close.json`
