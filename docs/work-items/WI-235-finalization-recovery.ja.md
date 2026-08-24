---
author: AI Cockpit maintainers
title: "WI-235 — finalization recovery と clean-batch boundary"
workItemId: WI-235-finalization-recovery
description: "WI-234 の archived delivery を recovery し、verification と archive の前に reviewed PR context を bind します。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-235-finalization-recovery
---

# WI-235 — finalization recovery と clean-batch boundary

WI-235 は PR #185 で判明した process defect のための狭い successor です。WI-234 が
`finalize-plan` の前に archive されたため、archived Contract の resource context が
pending のままとなり、governance gate は terminal decision の欠落を正しく拒否しました。
失敗した PR と WI-234 の全 bytes は immutable のまま保持します。

この successor は verification 前に実際の reviewed PR context を bind し、recovery
decision を記録してから通常の finalization boundary を完了します。次の batch の開始時に
WI-234/WI-235 の obsolete worktree や branch が残らないことも確認します。

## Acceptance boundary

- `stale_awaiting_merge_close` regression は fail closed のままです。
- WI-234 は正確な recovery receipt で Recovered として参照します。
- `finalize-plan` を verify、finish、archive より先に実行します。
- 並列 attach migration fixture は衝突しないパスを使い、workspace 全体のテストを並列実行でも決定的に保ちます。
- pre-merge finalization receipt、hosted checks、merge observation、exact cleanup、
  structured close を同じ PR head に bind します。

## References

- [Japanese parity ledger](../reference/reference-parity.ja.md)
- [WI-234 immutable Work Item](WI-234-post-merge-governance-cleanup.ja.md)
- [Governance gate](../../tests/ci/governance_integrity_gate.py)
