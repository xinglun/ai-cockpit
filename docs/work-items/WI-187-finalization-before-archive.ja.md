---
author: AI Cockpit maintainers
title: "WI-187 — archive 前の resource finalization plan"
description: "現行 Work Item の archive 前に、明示的で provisional ではない resource finalization plan を必須にします。"
audience:
  - maintainer
  - reviewer
workItemId: WI-187-finalization-before-archive
status: current
authority: canonical
lastVerifiedBy: WI-187-finalization-before-archive
---

# WI-187 — archive 前の resource finalization plan

WI-187 は lifecycle の順序上の欠落を修正します。`start` は意図的に
provisional な `resourceContext` を記録します。ローカル branch と worktree
の観測値は存在しますが、`baseBranch`、`baseRemote`、`provider`、
`pullRequest` は `unknown` のままです。この context は resource
finalization plan ではありません。

標準の `finish` 境界は `finish_ready` を生成する前に、欠落または
provisional な context を拒否します。`archive` も active な Contract、
Summary、Outcome、report、event、approach の bytes を移動する前に同じ条件を
独立して再確認します。operator は verification、finish、archive の前に
`work-item finalize-plan` を実行し、完全で検証済みの identity-bound context
を記録する必要があります。有効な non-provisional plan があれば、従来の
成功する lifecycle flow は維持されます。

## historical / recovery 境界

WI-186 は観測済み predecessor です。公開 v0.2.23 Runtime がその記録を
archive した時点で、Contract には `start` が書いた provisional context が
残っていました。WI-187 は historical archive bytes を編集、正規化、または
遡及的に finalization 済みへ昇格しません。historical reader は optional
context の読み取りを継続し、明示的な supersession recovery route は
predecessor artifacts を byte-for-byte で保持します。この recovery 例外には
独自の identity-bound recovery decision が必要であり、現行の通常 Work Item
が `finalize-plan` を迂回することはできません。

WI-187 は、この観測済み gap に対する bounded successor です。インストール
済み Runtime は `.ai/decisions/` に `supersede` recovery receipt を記録し、
WI-186 の正確な Contract、Summary、Outcome、events の digests を WI-187 に
厳密に binding します。この receipt は追記のみであり、WI-186 の結果を再解釈
せず、WI-186 archive bundle 内の file を一切書き換えません。

WI-187 の最初の実行自体も、この順序が強制される前に `finish_ready` へ到達
しました。そのため Runtime は verification 後の provisional plan の置換を
正しく拒否しました。これらの正確な record は digest-bound supersession で
保持され、`WI-190-finalization-plan-order` が正しい順序で lifecycle を再実行し、
検証済み implementation を引き継ぎます。

回帰 suite は protocol の provisional 判定、repository の archive 拒否と
active bytes 保持、CLI の拒否と recovery state、有効 plan 後の成功、
historical evidence の可読性、および superseded predecessor の不変 recovery
を検証します。共有 reference parity file は本 Work Item の対象外です。

[English](WI-187-finalization-before-archive.md) ·
[简体中文](WI-187-finalization-before-archive.zh-CN.md)
