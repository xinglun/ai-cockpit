---
author: AI Cockpit maintainers
title: "WI-593 — WI-592 documentation parity recovery"
description: "前置履歴を書き換えず、欠落した WI-592 parity 登録を再配信します。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-593-wi591-doc-parity-recovery
lastVerifiedBy: WI-593-wi591-doc-parity-recovery
---

[English](WI-593-wi591-doc-parity-recovery.md) · [简体中文](WI-593-wi591-doc-parity-recovery.zh-CN.md)

# WI-593 — WI-592 documentation parity recovery

## Objective

WI-592 が archive された後に documentation governance gate が検出した三言語 parity
登録漏れを再配信します。WI-592 の immutable record は保持し、現在の projection を
監査可能にします。

## Boundary

対象は三つの reference-parity ファイルと WI-592/WI-593 の英語・中国語・日本語ページ
だけです。Runtime behavior、Cargo metadata、release artifact、object repository、
global Agent/MCP configuration、historical `.ai` bytes は範囲外です。

## Acceptance

1. 三つの parity file に WI-592 の recovery row と WI-593 の pending row が一つずつ
   一貫して存在し、Work Item link と evidence boundary が有効であること。
2. WI-592 の archived Contract、evidence、recovery decision、event history が byte-for-byte
   変更されないこと。
3. 三言語ページの frontmatter が有効で、append-only recovery 関係を説明し、虚偽の
   terminal state を主張しないこと。
4. terminal close 前に `python3 tests/docs/promote_closed_work_item.py --repo <repository>
   --check-all`、`tests/docs/documentation_acceptance.sh`、`tests/docs/parity_status_check.sh`
   が成功すること。

## Verification

明示的な repository context で Contract に記載した documentation acceptance、parity、
status consistency、locked workspace check を実行し、merge 後に full documentation gate
を再実行します。
