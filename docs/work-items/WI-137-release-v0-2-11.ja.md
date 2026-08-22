---
author: AI Cockpit maintainers
workItemId: WI-137-release-v0-2-11
title: v0.2.11 の公開と immutable adopter acceptance
description: merge 済み Runtime 修正を公開し、隔離 adopter と current repository で公開 binary を検証する。
audience:
  - adopter
  - maintainer
status: release-preparation
authority: canonical
lastVerifiedBy: WI-137-release-v0-2-11
---

# WI-137 — v0.2.11 の公開と immutable adopter acceptance

## Intent

WI-135 の repository-bound retention/evidence 検証と WI-136 の Task Outcome
report を含む最初の immutable Runtime を公開し、公開 binary が新しい adopter と
この repository を治理できることを確認する。

## Scope と境界

- workspace と current release documentation を `v0.2.11` に更新する。
- source quality、release policy、fresh-adopter、N-1 acceptance を実行する。
- 公開済み v0.2.11 artifact だけを install し、current repository を検証する。
- release acceptance artifact を repository history と分離して保存する。

Runtime の新機能、historical evidence の書き換え、global Agent/MCP 設定、外部
Homebrew tap の変更は行わない。source/workspace binary を release acceptance の代用にしない。

## Acceptance

1. Cargo metadata、archive 名、manifest、三言語 route が v0.2.11 で一致し、過去の N-1 引用は明示的に残る。
2. fresh adopter は immutable v0.2.11 Release だけを download/検証し、
   `first-adopter-smoke = not_ready`、repository/runtime identity、evidence reuse、lifecycle、isolation、cleanup receipt を記録する。
3. N-1 acceptance は v0.2.10 → v0.2.11 の互換性、旧 bytes と Release truth の保持を確認する。
4. install 済み公開 binary が current repository の inspect、status、doctor、Agent doctor、人向け Outcome を通過し、WI-136 report を読める。

## Verification と evidence

workspace verification、公開 fresh-adopter、N-1 upgrade の各 receipt、Runtime
identity（version、archive digest、binary digest、target、download source）と最終 Outcome を保存する。
