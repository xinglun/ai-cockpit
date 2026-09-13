---
author: AI Cockpit maintainers
title: "WI-824 — verification の事前条件と永続化された attempt"
description: "実行開始前に無効な verification 入力を拒否し、安全な復旧のために実行 attempt を保持する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized
workItemId: WI-824-verification-preconditions
lastVerifiedBy: WI-824-verification-preconditions
terminalArchive: .ai/work-items/archive/WI-824-verification-preconditions.contract.json
terminalVerification: .ai/evidence/WI-824-verification-preconditions.verification.json
terminalDecision: .ai/decisions/WI-824-verification-preconditions.close.json
---

[English](WI-824-verification-preconditions.md) · [简体中文](WI-824-verification-preconditions.zh-CN.md)

# WI-824 — verification の事前条件と永続化された attempt

## Intent と boundary

この Work Item は、検証プロセスを起動する前に現在の preflight、Contract、repository identity を確認する。条件を満たさない場合は早期に停止する。また、実行済み各 node の bounded output、exit code、timeout 状態、経過時間、command identity、Runtime identity、source snapshot を正式な completion receipt とは独立して保持する。過去の evidence bytes は書き換えない。

## Recovery behavior

source snapshot、command と dependency input、Runtime digest、repository identity、Contract execution scope がすべて一致する場合だけ、成功した attempt を再利用できる。したがって governance projection だけの修正では同じ command の再実行を避けられるが、source、command、Runtime、関連 dependency の変更では再利用を無効にする。事前条件拒否は project process の spawn 数 0 と構造化診断を記録する。

## Acceptance evidence

- archive: `.ai/work-items/archive/WI-824-verification-preconditions.contract.json`
- formal verification: `.ai/evidence/WI-824-verification-preconditions.verification.json`
- attempt records: `.ai/evidence/WI-824-verification-preconditions.verification-attempt.*.json`
- close decision: `.ai/decisions/WI-824-verification-preconditions.close.json`
