---
author: AI Cockpit maintainers
title: WI-191 — Append-only finalization transition chain
workItemId: WI-191-finalization-transition-chain
description: "merge と cleanup を append-only transition として記録し、pre-merge evidence を不変に保持します。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-191-finalization-transition-chain
---

# WI-191 — Append-only finalization transition chain

WI-190 により、正当な pre-merge blocked canonical receipt が merge と cleanup 後に進めない欠陥が判明しました。WI-191 はその receipt を保持し、exact state continuity を持つ typed digest-addressed transition と一意な head resolver を追加します。merge observation と resource cleanup は別 transition であり、`finalize-verify` と `close` は最新 head を束縛します。foreign、stale、forked、malformed、symlinked、sequence-invalid chain は fail closed となり、legacy canonical receipt は互換性を維持します。
