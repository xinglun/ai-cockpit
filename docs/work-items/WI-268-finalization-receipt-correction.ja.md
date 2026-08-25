---
author: AI Cockpit maintainers
title: "WI-268 — Finalization receipt correction"
workItemId: WI-268-finalization-receipt-correction
description: "immutable な invalid pre-merge finalization receipt を明示的 successor で修正します。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-268-finalization-receipt-correction
authority: canonical
---

# WI-268 — Finalization receipt correction

## Intent

WI-267 は governance gate が拒否する worktree identity を持つ生成 pre-merge receipt のため、immutable な recovery history として保持します。本 successor は protocol-valid receipt を記録し、WI-267 を書き換えずに三言語 parity docs へ recovery 関係を投影します。

## Scope と evidence boundary

- successor Contract、branch、worktree、PR、repository、Runtime、archived Contract identity を正確に bind します。
- WI-267 の archive、verification、invalid finalization、recovery bytes は変更しません。
- 三言語 parity docs と本 Work Item docs を更新し、predecessor/successor 関係を明示します。
- hosted review、verification、finalization、exact cleanup、structured close の後だけ昇格します。

## Verification

- `cargo test --locked --workspace`
- `bash tests/docs/parity_status_check.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/work_item_status_consistency_test.sh`
- `--repo` を明示した installed Runtime の lifecycle と visible human Outcome

最終 handoff は可視の `Outcome: 🟢`、`Outcome: 🟡`、`Outcome: 🔴` のいずれかで始まり、status、unknowns、evidence、human decision、next action を含めます。
