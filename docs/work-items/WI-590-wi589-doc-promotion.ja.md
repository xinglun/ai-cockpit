---
author: AI Cockpit maintainers
title: "WI-590 — WI-589 終端ドキュメント昇格"
description: "WI-589 の verified close 後に documentation projection を昇格します。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-590-wi589-doc-promotion
lastVerifiedBy: WI-590-wi589-doc-promotion
---

[English](WI-590-wi589-doc-promotion.md) · [简体中文](WI-590-wi589-doc-promotion.zh-CN.md)

# WI-590 — WI-589 terminal documentation promotion

## Objective

WI-589 の immutable archive、verification、finalization、close evidence が
検証された後、三言語の Work Item と reference-parity projection を terminal
にします。本 WI は documentation projection だけを変更します。

## Boundary

Runtime behavior、object repository、global Agent/MCP configuration、生成済み
evidence/decision bytes は対象外です。Governance facts は immutable Runtime
records から導出します。

## Acceptance

1. WI-589 の英中日ページが terminal evidence path と Implemented status を
   保持し、意味内容を変更しない。
2. WI-589 の三つの parity row が対応する terminal evidence path を示す。
3. WI-590 自身のページと parity row を deterministic self-terminal promotion
   用に登録し、終了後に追加の documentation debt を作らない。

## Verification

明示的な repository context で
`python3 tests/docs/promote_closed_work_item.py --repo <repository> --check-all`、
`tests/docs/documentation_acceptance.sh`、`tests/docs/parity_status_check.sh` を実行します。
