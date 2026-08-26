---
author: AI Cockpit maintainers
title: "WI-298 — v0.2.32 リリース finalization recovery"
workItemId: WI-298-release-v0-2-32-finalization-recovery
description: "不変な archive を書き換えず、WI-297 に欠けていた reviewed resource-finalization chain を完了する。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-298-release-v0-2-32-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-298-release-v0-2-32-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-298-release-v0-2-32-finalization-recovery.verification.json
terminalFinalization: .ai/decisions/WI-298-release-v0-2-32-finalization-recovery.finalize.json
terminalDecision: .ai/decisions/WI-298-release-v0-2-32-finalization-recovery.close.json
authority: canonical
---

# WI-298 — v0.2.32 リリース finalization recovery

## Intent

WI-297 の archive 後に判明した `finalize-plan` 境界の欠落を recovery します。
先行 Work Item の archive、verification、recovery receipt、merge 済み PR は
不変のままにし、この Work Item は狭い closure recovery だけを記録します。

## Scope

- 正確な PR #258、branch、worktree、default branch context を bind します。
- installed Runtime で recovery record の verification と hosted quality checks
  を実行します。
- provider finalization を記録し、正確な cleanup を検証し、structured human
  close receipt を作成します。
- predecessor/successor 関係と全 evidence の identity binding を維持します。

## Out of scope

Release 実装、Runtime behavior、package metadata、adopter acceptance、
Homebrew 公開、過去 archive の書き換えはこの recovery の範囲外です。

## Acceptance

- WI-297 archive bytes は変更せず、recovery decision から参照される。
- `finalize-plan` は successor の verification と archive より前に記録される。
- reviewed successor PR の hosted checks が成功する。
- `finalize-verify` が structured close 前に正確な feature branch/worktree
  cleanup を証明する。
- 可視の Human Outcome に status、unknowns、evidence、decision、next action
  が含まれる。

## Verification

明示的な `--repo` を付けた installed Runtime、repository/documentation gate、
hosted quality checks、および完全な
`finalize-plan → finalize → finalize-verify → close` chain を使用します。

reviewed PR の hosted quality 結果は terminal evidence の一部です。verification
evidence が不足していた以前の pre-archive 実行は履歴上の失敗として保持し、再利用しません。
