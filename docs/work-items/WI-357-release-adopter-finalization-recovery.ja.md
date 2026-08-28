---
author: AI Cockpit maintainers
title: "WI-357 — Release adopter finalization recovery"
workItemId: WI-357-release-adopter-finalization-recovery
description: "不変な predecessor evidence を書き換えず、WI-356 の delivery を実際の provider context に再束縛します。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-357-release-adopter-finalization-recovery
predecessor: WI-356-release-adopter-script-order
capabilityClaims:
  - adopter_finalization_recovery
---

# WI-357 — Release adopter finalization recovery

[English](WI-357-release-adopter-finalization-recovery.md) · [简体中文](WI-357-release-adopter-finalization-recovery.zh-CN.md)

## Intent と boundary

WI-357 は WI-356 の明示的な recovery successor です。WI-356 は PR #321 として
merge 済みですが、archive は PR 作成前に生成されたため provisional な
`pending` resource context を保持しています。本 Work Item は新しい監査可能な
lifecycle で実際の provider binding と正確な cleanup を記録し、WI-356 の archive、
evidence、outcome bytes は書き換えません。

範囲は recovery decision、三言語の governance record、reviewed PR の resource
binding、finalization/close evidence に限定します。Runtime feature、adopter
harness、release version、provider automation は対象外です。

## Delivery と verification

- recovery receipt は predecessor の Contract、Summary、Outcome、Events digest を
  束縛し、WI-356 と PR #321 を明示的に参照します。
- verification evidence を記録する前に、専用 branch/worktree と reviewed PR に
  実際の GitHub context を束縛します。
- `finalize-verify` は merge 済み PR、同期済み default branch、正確な local/remote
  branch と worktree cleanup を証明してから structured close を許可します。
- すべての receipt は repository-bound で installed Runtime が生成し、predecessor
  の歴史 bytes は不変のまま保持します。

## Recovery boundary

不変な predecessor Contract に埋め込まれた provisional resource context は実際の
provider receipt に置換できません。そのため successor の明示的な recovery を使い、
fail-closed で review 可能にします。偽の URL を使ったり predecessor record を編集したりしません。
