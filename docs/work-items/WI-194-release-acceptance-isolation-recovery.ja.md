---
author: AI Cockpit maintainers
title: "WI-194 — Release acceptance isolation recovery"
description: "immutable な WI-194 recovery history を保持し、bounded な release isolation delivery を WI-195 に引き継ぎます。"
audience:
  - maintainer
  - reviewer
workItemId: WI-194-release-acceptance-isolation-recovery
status: historical
authority: canonical
lastVerifiedBy: WI-195-governance-recovery-gate
---

# WI-194 — Release acceptance isolation recovery

WI-194 は WI-193 の release-acceptance isolation 実装を保持しましたが、archive
evidence は source-built Runtime で生成され、resource context は存在しない provider
PR を参照していました。bytes は immutable なまま保持され、current Release proof
ではなく historical evidence です。

したがって WI-194 は completed ではなく recovered です。明示的な recovery receipt は
archive の Contract、Summary、Outcome、events、repository、Runtime identity を bind し、
同じ bounded delivery を WI-195 に移します。historical archive、evidence、published
Release truth は書き換えません。

Evidence: `.ai/evidence/WI-194-release-acceptance-isolation-recovery.verification.json`;
recovery: `.ai/decisions/WI-194-release-acceptance-isolation-recovery.recovery.json`;
archive: `.ai/work-items/archive/WI-194-release-acceptance-isolation-recovery.archive.json`。

[English](WI-194-release-acceptance-isolation-recovery.md) ·
[简体中文](WI-194-release-acceptance-isolation-recovery.zh-CN.md)
