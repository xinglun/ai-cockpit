---
author: AI Cockpit maintainers
title: "WI-246 — Pending parity merge-ref recovery"
workItemId: WI-246-pending-parity-merge-ref-recovery
description: "WI-244 delivery を recovery し、hosted merge ref が追加する authoritative decision に parity を束縛します。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-246-pending-parity-merge-ref-recovery
authority: canonical
---

# WI-246 — Pending parity merge-ref recovery

WI-244 は typed pending parity registry を提供し、PR #196 で immutable verified archive
に到達しました。push tree は通過しましたが、hosted PR merge ref には default branch に
新しく存在する authoritative な WI-243 close receipt も含まれます。3 つの WI-243 row は
pre-merge finalize receipt だけを列挙していたため、governance gate は結合 tree を正しく
拒否しました。

## Recovery boundary

- Runtime receipt `.ai/decisions/WI-244-pending-parity-registry.recovery.json` は正確な
  predecessor Contract、Summary、Outcome、Events digest を束縛します。
- WI-244 archive、verification、finalization、PR #196、hosted-run bytes は immutable です。
  WI-246 は predecessor を書き換えずにそれらを投影します。
- Contract base は `origin/main` の
  `3fd982560ee28563bfab69d414f60575f3b2894a` です。recovery bootstrap commit
  `3a5693a` は governance history であり、base の代用ではありません。
- Draft PR #197 の正確な branch/worktree context は checkpoint と実装より前に
  `finalize-plan` で束縛済みです。

## Acceptance

3 つの WI-243 row は pre-merge finalize path を保持し、close path を追加します。WI-244
は recovery receipt とともに Recovered と表示し、WI-246 は merge/close 前まで In progress
です。決定的 regression は base-plus-feature merge tree を構築します。base の close decision
が row にない場合は 3 つの `missing_parity_decision` となり、全 row に両 path があれば
pass します。pending registry の厳密な schema、identity、Git ancestry、symlink、lifecycle
check は変更しません。

## Verification

governance、pending-registry、manifest、route、documentation、parity の focused test 後に、
strict typed repository gate を実行します。Rustfmt、Clippy、full workspace suite も必須です。
Runtime v0.2.31 が最終 verification、可視 Human Outcome、archive、append-only finalization
を記録します。
