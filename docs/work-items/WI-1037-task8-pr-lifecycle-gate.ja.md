---
author: AI Cockpit maintainers
workItemId: WI-1037-task8-pr-lifecycle-gate
title: Task 8 PR lifecycle gate 修正
description: Task 8 closeout を妨げる PR lifecycle gate のみを修正する。Task 8 の merge と cleanup 後に Task 9 へ進み、release 前に停止する。
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-1037-task8-pr-lifecycle-gate
terminalArchive: .ai/work-items/archive/WI-1037-task8-pr-lifecycle-gate.contract.json
terminalVerification: .ai/evidence/WI-1037-task8-pr-lifecycle-gate.verification.json
terminalDecision: .ai/decisions/WI-1037-task8-pr-lifecycle-gate.close.json
---

[English](WI-1037-task8-pr-lifecycle-gate.md) · [简体中文](WI-1037-task8-pr-lifecycle-gate.zh-CN.md)

# WI-1037 — Task 8 PR lifecycle gate 修正

[PR #997](https://github.com/xinglun/ai-cockpit/pull/997) の [CI run 36377627838](https://github.com/xinglun/ai-cockpit/actions/runs/36377627838) で記録された merge 前の lifecycle failure を扱う限定 successor である。通常の archived Work Item は PR merge 後でなければ formal close できないため、CI は archive Contract を実際に追加した正確な PR merge checkout のみを認識する。

## 境界

- `pull_request` merge ref、event の base/head、2つの Git parent が一致し、diff が archived Contract を追加した場合のみ formal close 待ちを許可する。
- 古い archive、不一致または malformed な event、direct push、後続の未 close commit は引き続き block する。
- merge 後の formal close 要件を維持し、integration 前に close receipt を作成しない。
- 汎用 governance redesign、Task 9 migration、release、version 変更、tag、publication は対象外。

## Acceptance

1. 正確な PR が追加した archive のみ `awaiting_merge_close` にでき、event と Git identity が一致する。
2. 古い未 close archive と不一致または malformed な PR event は block される。
3. 一時的に許されるのは正確な default-branch merge のみであり、その後 close のない非 merge commit は block される。
4. WI-1036 の三言語 parity 行は archived Contract を参照し、完了を主張せず merge 待ちと示す。
5. integration 前に対象 test と exact-head hosted CI が通る。Task 8 の merge/cleanup 後、Runtime が `readyOnBase` を報告した場合のみ Task 9 を開始し、release 前に human review のため停止する。

現在は進行中。必須 scenario、local check、hosted check、integration は未完了であり、このページは通過を主張しない。
