---
author: AI Cockpit maintainers
description: "v0.2.10 を公開し immutable adopter acceptance を完了する。"
audience:
  - adopter
  - maintainer
authority: canonical
lastVerifiedBy: documentation-acceptance
workItemId: WI-120-release-v0-2-10
title: v0.2.10 の公開と immutable adopter acceptance
status: release-preparation
---

# WI-120 — v0.2.10 の公開と immutable adopter acceptance

## 目的

Contract の事前 Human Review gate を含む最初の公開 Runtime をリリースし、source fallback を使わず、
download した Release binary が新しい adopter を治理できることと直前版からの upgrade を証明します。

## 範囲

- workspace と current release 文書を `v0.2.10` に更新する;
- immutable な公開 artifact と Runtime identity を記録する;
- isolated root で fresh-adopter と v0.2.9 → v0.2.10 の N-1 acceptance を実行する;
- 公開 binary を install し、明示的な repository context で current repository を検証する。

## 境界

Runtime の新機能追加、historical evidence の書き換え、global Agent/MCP configuration の変更は行いません。
post-release acceptance の失敗は記録できますが、公開済み Release truth は書き換えません。

## 受入れ

- version、文書、release policy check が `v0.2.10` だけを current baseline とし、historical reference は明示的に残す;
- CI と release check が成功する;
- fresh-adopter で `first-adopter-smoke = not_ready` を維持し、download binary digest、repository identity、
  evidence reuse、lifecycle、isolation、cleanup receipt を記録する;
- N-1 acceptance が v0.2.9 → v0.2.10 の互換経路を証明する;
- install 済み公開 binary が `0.2.10` を報告し、current repository の inspect、status、doctor、Agent doctor、Outcome check が成功する。

## Evidence と decision の境界

Release の公開は adopter acceptance の証明ではありません。公開 archive、manifest、checksum、acceptance receipt、
Runtime identity はそれぞれ検証可能でなければなりません。yellow または red には Human decision が必要で、
acceptance criteria は Contract の原文として保持し、治理上の事実へ暗黙に翻訳しません。
