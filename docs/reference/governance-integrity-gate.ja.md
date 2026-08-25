---
author: AI Cockpit maintainers
title: Governance integrity gate
description: "current Work Item、evidence、terminal decision、文書 binding を fail-closed で動的に点検します。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-198-governance-gate-default-branch-discovery
---

# Governance integrity gate

`tests/ci/governance_integrity_gate.py` は固定 ID リストではなく repository record
から Work Item を動的に検出します。current release cycle、evidence identity、terminal
decision、Outcome、三言語 parity binding を検査し、finding は決定的に生成されます。
不完全な状態は fail-closed です。

## Recovery は完了ではない

正当な `.ai/decisions/<WI>.recovery.json` を持つ predecessor は
`lifecycleState: recovered` として報告されます。receipt は predecessor、空でない
successor、repository identity、reason、evidence refs を束縛しなければなりません。
Recovered predecessor は赤/blocked の不変 Outcome を保持できますが、緑へ昇格されず、
merge や release の承認ともみなされません。

欠落、malformed、foreign、binding 不足の recovery receipt は引き続き error です。
successor は独立して Contract、evidence、Outcome、parity、terminal decision を通過
しなければなりません。

## detached pull-request checkout

Hosted pull request job は、`refs/remotes/origin/HEAD` や event の base branch metadata
を持たない detached merge checkout で実行されることがあります。その場合、gate は不変な
Contract の `resourceContext.baseBranch` だけを狭い default branch fallback として使います。
receipt と Contract の resource context が完全一致する場合だけ受理し、repository、PR
URL/number、provider、remote、branch、worktree、base/head revision、runtime、evidence、
Contract digest の検査はすべて必須です。外部 event または remote が別の base branch を示す
場合、receipt は拒否されます。identity の欠落や矛盾は引き続き fail-closed です。

## Finalization head binding

`feature_branch` と `pull_request` phase では、pre-merge finalization receipt の
branch、pull request、worktree head が reviewed checkout head に解決できる場合だけ有効
です。後続 checkout で許されるのは canonical finalization transition または明示的に
allow-list された同一 Work Item の governance record の bounded append だけです。code、
test、無関係な evidence、その他の repository 変更があれば新しい receipt を要求し、
fail-closed とします。pending parity registry は、三言語 parity row の完了前に closed
Work Item を可視のままにするための、明示的に許可された repository-level governance append
です。

## 動的な Work Item 文書 projection

Contract または active Summary が `docs/reference/reference-parity*` または parity
registration を明示的に所有する current Work Item は parity/documentation Work Item です。
light gate は verification、archive、close の前に `docs/work-items/<WI>.md`、
`docs/work-items/<WI>.ja.md`、`docs/work-items/<WI>.zh-CN.md` の三つを regular な
non-symlink projection として要求します。frontmatter は有効で `workItemId` を束縛しなければ
ならず、生成済み `.ai` history は書き換えません。

同じ検査は current-cycle の archived parity Work Item にも適用されます。欠落、malformed、
foreign、symlink の文書は fail-closed です。通常の code Work Item は `active_non_parity` のまま
で、文書作成を強制されません。この方針は動的で、repository-bound gate を通じて adopter
repository にも継承されます。Rust Runtime のコピーは不要です。

この gate は verification tier や assurance を選択しません。risk/stage/policy による
選択と reference source の逐文件 conformance は別の検証境界であり、この inventory から
推測してはなりません。
