---
author: AI Cockpit maintainers
title: "WI-322 — lifecycle entry safety"
workItemId: WI-322-lifecycle-entry-safety
description: "repository の closure または start 前の base 条件が未解決なら fail-closed にする。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-322-lifecycle-entry-safety
terminalArchive: .ai/work-items/archive/WI-322-lifecycle-entry-safety.contract.json
terminalVerification: .ai/evidence/WI-322-lifecycle-entry-safety.verification.json
terminalFinalization: .ai/decisions/WI-322-lifecycle-entry-safety.finalize.json
terminalDecision: .ai/decisions/WI-322-lifecycle-entry-safety.close.json
---

# WI-322 — lifecycle entry safety

## Intent と boundary

有効な close decision がない archived Work Item、start 前の非 governance 変更、detached
branch、または既知の branch/base 不一致がある場合、新しい governed Work Item の開始を拒否します。
判定できない repository metadata は `unknown` のまま保持し、green readiness として表示しません。

チェックは repository ごとに分離され、process-global current project は作りません。明示的な
recovery continuation は既存の recovery path を使います。

## Scope と acceptance

- `work-item new` と `start` は未解決の archived closure を fail-closed で拒否し、immutable archive bytes を保持します。
- `status` は deterministic な `readiness`/`readyOnBase` と blockers を表示します。
- start 前のユーザー変更を拒否し、Runtime-owned `.ai` の書き込みは許可します。
- network access なしで discoverable remote default ref を検査し、metadata 不在時は readiness を `unknown` にします。
- 2 つの repository context を分離し、三言語の command/Agent workflow documentation を同期します。

## Verification

locked workspace test、lifecycle-entry regression、documentation gate、hosted CI で検証します。
すべての repository-bound Runtime command は明示的な `--repo` path を使います。

[English](WI-322-lifecycle-entry-safety.md) ·
[简体中文](WI-322-lifecycle-entry-safety.zh-CN.md)
