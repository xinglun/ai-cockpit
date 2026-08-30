---
author: AI Cockpit maintainers
title: "WI-427 — parity governance recovery"
description: ホスト CI が登録漏れを検出したため、recovery binding と三言語 parity ledger を再配信する。
workItemId: WI-427-parity-governance-recovery
audience: [contributor, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-427-parity-governance-recovery
---

# WI-427 — parity governance recovery

この successor は不変の recovery 履歴を保持しながら binding を再配信し、各
reference-parity ledger に選択された decision と evidence のパスを登録します。
predecessor の archive bytes は書き換えず、documentation gate も弱めません。

parity 行は archive 前の登録です。verification、レビュー済み merge、finalization、
close の receipt が揃った後にだけ `Implemented` になります。

[English](WI-427-parity-governance-recovery.md) · [中文](WI-427-parity-governance-recovery.zh-CN.md)
