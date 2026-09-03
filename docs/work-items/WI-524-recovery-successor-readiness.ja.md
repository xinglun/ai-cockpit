---
author: AI Cockpit maintainers
title: "WI-524 — recovery successor readiness entry-gate binding"
description: "検証済み recovery successor がある場合だけ archived predecessor blocker を抑制します。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
workItemId: WI-524-recovery-successor-readiness
lastVerifiedBy: WI-524-recovery-successor-readiness
terminalArchive: .ai/work-items/archive/WI-524-recovery-successor-readiness.contract.json
terminalVerification: .ai/evidence/WI-524-recovery-successor-readiness.verification.json
terminalFinalization: .ai/decisions/WI-524-recovery-successor-readiness.finalize.cab9a20e63481aea75e8801ff86a94cec5ddc4c99fe9602500b43537567272c6.json
terminalDecision: .ai/decisions/WI-524-recovery-successor-readiness.close.json
---

[English](WI-524-recovery-successor-readiness.md) · [简体中文](WI-524-recovery-successor-readiness.zh-CN.md)

## Goal

successor が repository-bound、manifest 検証済み、verified、明示的に close 済みの場合だけ、predecessor は repository entry gate を通過できます。

## Scope と Acceptance

- missing、stale、foreign、malformed、symlink、未 close の successor は fail closed のままにします。
- isolation 回帰テストと三言語 workflow/parity projection を追加し、historical evidence、object repository、global configuration は変更しません。
- 有効な terminal successor だけが対応する predecessor blocker を解除し、Rust・文書・governance・hosted CI が pass します。

## Verification

```text
cargo test --locked -p cockpit-repository --test lifecycle_entry --test recovery_decision -- --test-threads=1
cargo clippy --workspace --all-targets --all-features -- -D warnings
python3 tests/ci/governance_integrity_gate.py --repo <repo>
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
```
