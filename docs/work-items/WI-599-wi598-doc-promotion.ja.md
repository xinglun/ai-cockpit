---
author: AI Cockpit maintainers
title: "WI-599 — WI-598 終端ドキュメント昇格"
description: "三言語の parity evidence を先に登録し、検証済み WI-598 のドキュメント投影を昇格する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-599-wi598-doc-promotion
lastVerifiedBy: WI-599-wi598-doc-promotion
terminalArchive: .ai/work-items/archive/WI-599-wi598-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-599-wi598-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-599-wi598-doc-promotion.finalize.eacd2ab5f9639f57f01f2caabefc3f22aaaf2e7842260629b1a8d8d538903a67.json
terminalDecision: .ai/decisions/WI-599-wi598-doc-promotion.close.json
---

[English](WI-599-wi598-doc-promotion.md) · [简体中文](WI-599-wi598-doc-promotion.zh-CN.md)

# WI-599 — WI-598 終端ドキュメント昇格

## 目的

WI-598 の immutable archive、verification、finalization、close receipt が
有効になった後、その三言語 Work Item と reference-parity 投影を昇格する。
新しい verification evidence を生成する前に本 WI 自身の投影を登録し、
governance-integrity gate が lifecycle 全体を監査できるようにする。

## 境界

本 WI はドキュメント投影だけを変更する。Runtime 挙動、object repository、
global Agent/MCP configuration、source implementation、生成済み evidence
または decision bytes は対象外である。Contract の acceptance は作成時の
言語を authoritative とする。

## 受入れ

1. 三つの WI-598 Work Item ページが immutable な archive、verification、
   finalization、close receipt から導出された終端 path を含む。
2. 三つの reference-parity 行が WI-598 を Implemented とし、対応する
   終端 evidence path を含む。
3. verification evidence を生成する前に WI-599 の記録と三つの parity 行を
   登録し、close 後にだけ昇格する。
4. governance fact、source implementation、object repository、生成済み
   receipt bytes を変更しない。

## 検証

明示的な repository context で
`tests/docs/promote_closed_work_item.py --check`、
`tests/docs/documentation_acceptance.sh`、
`tests/docs/parity_status_check.sh`、reference inventory/metadata regression、
locked workspace checks を実行する。
