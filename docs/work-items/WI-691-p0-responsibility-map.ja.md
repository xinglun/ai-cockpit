---
author: AI Cockpit maintainers
title: “WI-691 — P0 アーキテクチャ責任マップ”
description: “アーキテクチャ最適化の現在の責任・依存境界を記録する。”
audience: [contributor, maintainer, reviewer]
status: in_progress
authority: human:repository-owner
workItemId: WI-691-p0-responsibility-map
lastVerifiedBy: WI-691-p0-responsibility-map
---

[English](WI-691-p0-responsibility-map.md) · [简体中文](WI-691-p0-responsibility-map.zh-CN.md)

# WI-691 — P0 アーキテクチャ責任マップ

## Intent

後続の architecture Work Item に先立ち、現在の source citation に基づいて observation、
governance、lifecycle、evidence、execution、persistence/recovery、status/Outcome
projection の責任境界を記録する。

## Boundary

Documentation-only。Rust source、test、公開 protocol、governance rule、`.ai/` record、
historical evidence は変更しない。基準は最新 remote default の revision
`99f7d2323ffb59b1e3edd6c1c833b00f50fb9698` である。

## Acceptance

- English、簡体中文、日本語の map が、現在の CLI/MCP entry point と observation、
  Contract/policy/evidence、governance、lifecycle、execution、projection、
  persistence/recovery の call chain を引用する。
- 各責任について authoritative fact、許可された I/O、validation/decision/display の owner、
  再利用できる mechanism を示す。
- concrete な重複・混在と未調査の問いを分け、struct や single-file rename から atomic
  snapshot/multi-file transaction を推論しない。
- P1-B、P2-A、P2-B、P2-C、P3 に bounded problem、target boundary、compatibility risk、
  verification method を記録する。
- 三言語 page と parity projection を一致させ、code/runtime behavior は変更しない。

## Evidence and verification

Runtime の必須 evidence は locked workspace test。documentation acceptance、parity status、
`git diff --check`、hosted governance checks は追加の delivery evidence とする。最終 state は
この page ではなく Runtime の archive、verification、finalization、close record から導く。

## Follow-up

現在の source には Git snapshot、typed protocol facts、capability-scoped evidence store、
lifecycle lock、bounded execution、final human renderer の分離がある。end-to-end observation
context の owner、multi-file lifecycle の authoritative commit record、reusable verification
と physical execution の境界は未調査であり、本 Work Item では実装しない。

