---
author: AI Cockpit maintainers
title: "WI-210 — v0.2.26 immutable release と adopter acceptance"
description: "merge 済み default branch から v0.2.26 を公開し、public binary で release governance transition を close する。"
audience:
  - maintainer
  - adopter
  - reviewer
workItemId: WI-210-release-v0-2-26
status: current
authority: canonical
lastVerifiedBy: WI-210-release-v0-2-26
---

# WI-210 — v0.2.26 immutable release と adopter acceptance

この Work Item は、失敗した v0.2.25 の公開履歴を不変のまま保持し、その後の
次の immutable public Release を確立します。version consistency、merge 済み PR と
release-tag proof、public binary の adopter/upgrade acceptance、WI-209 の installed
Runtime finalization と structured close を結び付けます。

Adopter boundary では download した public Release asset だけを使用します。source
checkout、`cargo build`、`cargo run`、workspace binary、local `target` artifact は
fallback として認めません。`v0.2.25` は immutable な failed history として保持し、
移動・再利用しません。

## Acceptance

1. release verification 前に v0.2.26 version、distribution document、tri-language
   parity が一致する。
2. 有効な premerge-finalize receipt と release-tag ancestor proof を持つ merge 済み
   commit に対してだけ immutable tag を作成する。
3. public adopter と N-1 upgrade acceptance は download asset のみで実行し、
   repository/runtime identity と isolation evidence を残す。
4. success、failure、interruption の全経路で temporary acceptance root を削除し、
   cleanup 結果を checksummed receipt に含める。
5. installed public Runtime が WI-209 の finalize、finalize-verify、structured human
   close を完了し、visible な localized Outcome handoff を出力する。

## Out of scope

Reference source の file-by-file parity 拡張は次の batch です。本 WI は無関係な
Runtime feature や user-global Agent/MCP configuration を変更しません。

## Evidence boundary

公開 Release と download した archive/manifest は immutable external evidence です。
post-release failure は `releasePublished: true` と `adopterAcceptance: failed` を
記録し、公開済み Release の truth を書き換えません。
