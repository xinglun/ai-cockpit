---
author: AI Cockpit メンテナー
title: "WI-842 — custom evidence verification preconditions"
description: "project verification process の起動前に repository-bound custom evidence を検証する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: authorized
workItemId: WI-842-custom-evidence-precondition
lastVerifiedBy: WI-842-custom-evidence-precondition
terminalArchive: .ai/work-items/archive/WI-842-custom-evidence-precondition.contract.json
terminalVerification: .ai/evidence/WI-842-custom-evidence-precondition.verification.json
terminalDecision: .ai/decisions/WI-842-custom-evidence-precondition.close.json
---

[English](WI-842-custom-evidence-precondition.md) · [简体中文](WI-842-custom-evidence-precondition.zh-CN.md)

# WI-842 — custom evidence verification preconditions

## Intent と boundary

この Work Item は verification entry gate が project process を起動する前に、
current repository、Contract、file type、byte digest を使って custom evidence を
検証するようにします。complete な projection は report-only warning により拒否
されません。一方、missing、stale、malformed、foreign、symlink、non-regular の
evidence は引き続き fail-closed です。Runtime 全体の reuse policy、historical
evidence の書き換え、release publication、resource cleanup は対象外です。

## Recovery boundary

同じ範囲の failure は current Work Item を amend して再検証します。scope、authority、
base が本当に異なる場合、独立した変更、安全でない in-scope repair、immutable な
failed delivery、または明示的な human direction の場合だけ successor を使います。
successor は predecessor binding を保持しなければなりません。

## Acceptance

- current repository に bind された complete custom evidence は verification precondition を通過する。required な high-risk scenario が未実行でも、計画が宣言されていればよい。
- invalid custom evidence は project process の spawn 前に stable diagnostic 付きで拒否される。
- formal completion evidence の記録に失敗しても、attempt の exit status、timeout、duration、logs、input identity は保持される。
- governance-only correction では execution input が同一なら reuse でき、execution input の変更では reuse が失効する。

## Verification

- `cargo test --locked -p cockpit-repository --test lifecycle_entry`
- `cargo test --locked -p cockpit-repository --test archive_integrity --test verification_attempts`
- `cargo fmt --all -- --check`
- `git diff --check`
