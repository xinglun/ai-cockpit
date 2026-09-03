---
author: AI Cockpit maintainers
title: "WI-524 — recovery successor readiness entry-gate binding"
description: "証明されていない recovery successor が repository-wide pending-close blocker を抑制しないようにする。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-524-recovery-successor-readiness
lastVerifiedBy: WI-524-recovery-successor-readiness
---

[English](WI-524-recovery-successor-readiness.md) · [简体中文](WI-524-recovery-successor-readiness.zh-CN.md)

## Goal

Repository readiness を検証済み recovery successor に結び付けます。successor が repository-bound、manifest 検証済み、verified、明示的に close 済みの場合だけ predecessor は entry gate を通過できます。

## Scope

- archived predecessor の `pending close` blocker を抑制する前に recovery successor lineage を検証する。
- missing、stale、foreign、malformed、symlink、未 close の successor は fail closed のままにする。
- repository isolation の回帰テストと三言語の workflow/parity 文書を追加する。
- historical evidence を不変に保ち、object repository と global Agent/MCP configuration は変更しない。

## Acceptance

- 有効で close 済みの terminal successor だけが対応する predecessor の pending-close blocker を解除する。
- 無効または不完全な successor は blocker として残る。
- 並列 repository の isolation を維持し、既存 lifecycle を回帰させない。
- Rust tests、documentation acceptance、governance integrity、hosted CI がすべて pass する。
- Runtime が生成した evidence と historical archive bytes を手編集しない。

## Verification

```text
cargo test --locked -p cockpit-repository --test lifecycle_entry --test recovery_decision -- --test-threads=1
cargo clippy --workspace --all-targets --all-features -- -D warnings
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
python3 tests/docs/work_item_status_consistency.py --repo <repo>
bash tests/docs/documentation_acceptance.sh
python3 tests/ci/governance_integrity_gate.py --repo <repo>
git diff --check
```
