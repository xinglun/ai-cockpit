---
author: AI Cockpit maintainers
title: "WI-835 — lifecycle cleanup disposition"
description: "残存する release branch と worktree の証拠付き disposition を記録する。"
audience: [maintainer, reviewer]
status: implemented
authority: authorized
workItemId: WI-835-lifecycle-cleanup
lastVerifiedBy: WI-835-lifecycle-cleanup
terminalArchive: .ai/work-items/archive/WI-835-lifecycle-cleanup.contract.json
terminalVerification: .ai/evidence/WI-835-lifecycle-cleanup.verification.json
terminalDecision: .ai/decisions/WI-835-lifecycle-cleanup.close.json
---

[English](WI-835-lifecycle-cleanup.md) · [简体中文](WI-835-lifecycle-cleanup.zh-CN.md)

# WI-835 — lifecycle cleanup disposition

## Intent と boundary

WI-835 は、現在の non-main remote release branch と local worktree の各項目について、
証拠に結び付いた disposition を記録します。dirty、divergent、未 merge、曖昧な
binding、または evidence 不足の resource は保持し、正確な reviewed merge、Runtime
lifecycle、clean resource の事実がすべて確認できた場合だけ削除します。

Runtime が生成した lifecycle record と historical evidence は immutable のまま保持します。
release tag、Release、product behavior、無関係な source change は範囲外です。

## Acceptance

- 残りの remote branch 全件に、PR state、merge fact、worktree state、blocker または
  cleanup evidence を含む disposition がある。
- Runtime 生成の archive、finalization、close record を使い、historical evidence bytes
  を書き換えたり削除したりしない。
- finalization verification 後に、正確に一致する clean な merged branch と worktree だけを削除する。
- cleanup Work Item が reviewed PR integration、archive、close、documentation promotion
  checks を通過する。

## Verification

- repository-bound disposition verifier による Runtime `verify`。
- `git ls-remote --heads origin 'codex/*'` と local `git worktree list`。
- 全 branch の GitHub PR state と merge fact の照合。
- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`。
