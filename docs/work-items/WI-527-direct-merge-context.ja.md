---
workItemId: WI-527-direct-merge-context
title: "WI-527 — direct-merge recovery context 互換"
status: implemented
mode: code
author: AI Cockpit maintainers
description: "archived local resource context を保持する歴史的 direct-merge receipt の限定互換。"
audience:
  - maintainer
  - adopter
authority: canonical
lastVerifiedBy: WI-527-direct-merge-context
terminalArchive: .ai/work-items/archive/WI-527-direct-merge-context.contract.json
terminalVerification: .ai/evidence/WI-527-direct-merge-context.verification.json
terminalFinalization: .ai/decisions/WI-527-direct-merge-context.finalize.f7bc389eb8064f2451fb5cbd0bb28785546030040c999d25e65f6e0adb5a7c85.json
terminalDecision: .ai/decisions/WI-527-direct-merge-context.close.json
---

# WI-527 — direct-merge recovery context 互換

## 意図と境界

archived Contract が元の local `resourceContext` を保持している repository
でも、PR のない歴史的 merge recovery を使えるようにします。Runtime がこの
context を受け付けるのは明示的な `direct_merge_no_pr` / `historical_low`
receipt に限定し、repository、Work Item、branch、worktree、base、実際の merge
commit と parents を引き続き bind します。PR 番号の捏造や object repository の
変更は行いません。

## 実装

- protocol はこの限定された歴史ケースで unchanged な archived local context
  を受け付け、foreign な branch/worktree/base identity は拒否します。
- `finalize-recovery-plan` は provider/URL を推測せずに済む、identity-consistent
  な historical context を出力します。
- Rust protocol/repository regression は unchanged/transformed context と実際の
  Git parent binding を検証します。

## 受け入れ

公開 Runtime は正直な first direct-merge record を受け付け、malformed、foreign、
stale、symlink、non-ancestor 入力には fail-closed である必要があります。歴史 bytes
を変更せず、targeted/workspace test、documentation check、通常の lifecycle
evidence を通過させます。

## object repository handoff

`/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator` は本 WI では
read-only です。release 後、object team が `finalize-recovery-plan` を再実行し、
公開版の suggested receipt だけを適用します。
