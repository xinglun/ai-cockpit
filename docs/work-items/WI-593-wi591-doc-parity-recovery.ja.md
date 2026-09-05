---
author: AI Cockpit maintainers
title: "WI-593 — WI-592 parity recovery 再検証"
description: "WI-592 の履歴を書き換えず、append-only successor で不足していた parity 登録を再配信します。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-593-wi591-doc-parity-recovery
lastVerifiedBy: WI-593-wi591-doc-parity-recovery
terminalArchive: .ai/work-items/archive/WI-593-wi591-doc-parity-recovery.contract.json
terminalVerification: .ai/evidence/WI-593-wi591-doc-parity-recovery.verification.json
terminalFinalization: .ai/decisions/WI-593-wi591-doc-parity-recovery.finalize.f1adebd4711b32d6c621eb97a93219fe741ed770d738229ff4e65c712b470b4b.json
terminalDecision: .ai/decisions/WI-593-wi591-doc-parity-recovery.close.json
---

[English](WI-593-wi591-doc-parity-recovery.md) · [简体中文](WI-593-wi591-doc-parity-recovery.zh-CN.md)

# WI-593 — WI-592 parity recovery 再検証

## Objective

WI-592 recovery decision で判明した三言語 parity 登録不足を再配信し、現在の
verification evidence を生成します。WI-592 の archive、Contract、Summary、Outcome、
履歴 verification bytes は immutable のまま保持します。

## Boundary

この successor は三つの reference-parity projection と自身の documentation/evidence
だけを扱います。Runtime behavior、release artifact、object repository、global Agent/MCP
configuration、および WI-592 の immutable bytes は対象外です。

## Acceptance

1. 最新の reviewed `main` で三言語 parity gate が成功し、WI-592 の archive/evidence
   bytes を書き換えないこと。
2. Recovery decision が WI-592 の repository identity と immutable digest に束縛されること。
3. Verification と documentation が未裏付けの完了・ガバナンス判断を生成しないこと。

## Verification

明示的な repository context で `cargo test --locked --workspace`、
`tests/docs/parity_status_check.sh`、
`python3 tests/docs/work_item_status_consistency.py --repo <repository>`、
`tests/docs/documentation_acceptance.sh` を実行します。
