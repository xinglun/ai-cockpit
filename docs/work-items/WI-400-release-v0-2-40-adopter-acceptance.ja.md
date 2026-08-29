---
author: AI Cockpit maintainers
title: "WI-400 — v0.2.40 公開 Release adopter 受入れ"
description: "隔離した adopter で不変の v0.2.40 Release binary をゼロから検証する。"
workItemId: WI-400-release-v0-2-40-adopter-acceptance
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-400-release-v0-2-40-adopter-acceptance
capabilityClaims: [release_acceptance, repository_isolation, evidence_reuse]
---

# WI-400 — v0.2.40 公開 Release adopter 受入れ

[English](WI-400-release-v0-2-40-adopter-acceptance.md) · [简体中文](WI-400-release-v0-2-40-adopter-acceptance.zh-CN.md)

## Intent

不変の公開 v0.2.40 Release が新規 adopter をゼロから治理できることを、
Runtime identity、evidence reuse、lifecycle 記録、global-root isolation と共に監査可能にする。

## Boundary

対象は post-release artifact acceptance、temporary adopter と cleanup receipt、
closed WI-399 projection の昇格、生成された acceptance evidence の保持だけである。
Runtime semantics、reference parity、business-project code、global Agent/MCP configuration は変更しない。
Harness は source build に fallback してはならない。

## Acceptance

1. 公開 v0.2.40 archive と binary を Release から取得し、manifest と SHA-256 identity を照合する。
2. 新規 adopter に隔離 scaffold と独立 repository identity を作成し、人間入力が揃うまで
   `first-adopter-smoke` を `not_ready` に保持する。
3. schema-2 evidence、正確な reuse/re-execution、structured close decision、Runtime identity を記録する。
4. HOME/XDG は変更せず、隔離 Runtime write roots を記録し、receipt 後に一時 run root を削除する。

## Verification boundary

被測定 Runtime は公開 Release のみ。Acceptance artifacts は Release truth や履歴 record を書き換えない。
