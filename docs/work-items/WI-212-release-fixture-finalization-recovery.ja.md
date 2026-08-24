---
author: AI Cockpit maintainers
title: "WI-212 — WI-211 finalization recovery"
description: "immutable な WI-211 archive を回復し、必要な PR resource-finalization 順序を戻す。"
audience:
  - maintainer
  - reviewer
workItemId: WI-212-release-fixture-finalization-recovery
status: implemented
authority: canonical
lastVerifiedBy: WI-212-release-fixture-finalization-recovery
---

# WI-212 — WI-211 finalization recovery

WI-211 は PR #160 の context を `finalize-plan` で bind する前に verification と archive
まで進みました。Installed Runtime は verification evidence が既に記録されているため、
後続の finalization を正しく拒否しました。この successor は WI-211 の immutable recovered
history を保持し、bytes を書き換えずに不足していた resource-finalization boundary を戻します。

## Acceptance

1. strict successor recovery receipt で WI-211 を bind し、WI-211 を immutable のまま保持する。
2. この successor の verification 前に PR #160 の resource context を bind し、merge 前の receipt は
   `awaiting_merge_close` とする。
3. WI-211 が recovered、WI-212 が merge close 待ちの状態で governance gates が成功する。
4. hosted merge、正確な cleanup、finalize-verify、structured human decision の後だけ WI-212 を close する。

## Out of scope

WI-211 record の書き換え、v0.2.26 の移動、reference source の file-by-file 比較、
user-global Agent/MCP configuration は対象外です。
