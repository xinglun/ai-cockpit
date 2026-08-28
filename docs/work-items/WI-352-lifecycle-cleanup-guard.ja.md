---
author: AI Cockpit maintainers
title: "WI-352 — lifecycle cleanup guard"
workItemId: WI-352-lifecycle-cleanup-guard
description: "未完了の lifecycle cleanup を可視化し、repository と release-adopter 実行を fail-closed にする。"
audience: [maintainer, reviewer]
status: in_progress
authority: translation
canonical: docs/work-items/WI-352-lifecycle-cleanup-guard.md
lastVerifiedBy: WI-352-lifecycle-cleanup-guard
terminalArchive: .ai/work-items/archive/WI-352-lifecycle-cleanup-guard.contract.json
terminalVerification: .ai/evidence/WI-352-lifecycle-cleanup-guard.verification.json
capabilityClaims: [lifecycle_governance, cleanup_handoff]
---

# WI-352 — lifecycle cleanup guard

[English](WI-352-lifecycle-cleanup-guard.md) · [简体中文](WI-352-lifecycle-cleanup-guard.zh-CN.md)

## Intent と boundary

close の証拠が欠落または無効な archived Work Item を明確に non-terminal として扱います。
Runtime は status と human Outcome に正確な cleanup/finalization/close の次アクションを
表示し、repository-local state と共有 Runtime の境界を保ちます。release-adopter harness
は receipt を保存した後、成功・失敗の両方で隔離 run root を削除し、cleanup によって
acceptance truth を書き換えません。

## Verification

- archived だが未 close の状態は blocking/yellow であり、green として報告せず、次の
  Work Item を許可しません。
- 有効な finalization と close は reviewed PR、branch、worktree、repository、Runtime
  identity に bind されます。
- harness/wrapper の成功・失敗 cleanup をテストし、HOME/XDG_CONFIG_HOME は書込み禁止、
  TMPDIR/CARGO_HOME は隔離された Runtime write root として扱います。
- 英語・簡体字中国語・日本語の文書で同じ境界を示し、immutable な archive/evidence を保全します。

## Delivery status

実装と verification evidence は archive 済みです。review 中の PR は provider finalization、
正確な resource cleanup、structured close decision を完了してから terminal になります。
