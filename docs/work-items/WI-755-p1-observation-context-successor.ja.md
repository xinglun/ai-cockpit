---
author: AI Cockpit maintainers
title: "WI-755 — P1 観察コンテキスト successor"
description: "現在の default branch から明示的な観察コンテキスト境界を完了する。"
audience: [contributor, maintainer, reviewer]
workItemId: WI-755-p1-observation-context-successor
status: recovered
authority: human:repository-owner
lastVerifiedBy: WI-758-wi755-finalization-recovery
recoveryDecision: .ai/decisions/WI-755-p1-observation-context-successor.recovery.ccd6ce5cf8f1c2a563578437cc313079462d08a88b04fb8b61b734f54bf237a2.json
---

[English](WI-755-p1-observation-context-successor.md) · [简体中文](WI-755-p1-observation-context-successor.zh-CN.md)

# WI-755 — P1 観察コンテキスト successor

## Intent

現在の `origin/main` から残る P1-B のアーキテクチャ境界を完了する。失敗した
PR #697 と PR #698 は復活させない。一つの governance judgment は一つの明示的な
request-scoped、phase-scoped observation context を消費し、下位 helper が暗黙に
repository を再観察しない。

## Boundary

`RepositoryExecutionContext` は `before_governance`、`after_execution`、
`before_persistence` のために `ObservationContext` を生成する。context は repository
identity、任意の Runtime identity、Contract の raw/model identity、source snapshot digest、
governance configuration/policy identity、parsed observation、project-governance facts、
unknowns、consistency を束ねる。`validate_current` は新しい境界チェックを行い、source、
attached identity、Contract、governance configuration の変更時に古い context を拒否する。
phase は別の lifecycle phase に再利用できない。

preflight の governance path は context を消費し、解析済みの project-governance facts を再利用する。
互換 wrapper と既存 protocol bytes は維持する。

## Compatibility and limits

新しい crate、trait、global cache、protocol field、governance rule、Outcome wording、lifecycle
transition、physical-execution policy は追加しない。context は transaction ではなく、複数ファイルの
read を atomic snapshot とみなさない。execution 後または persistence 前には新しい phase context を取得する。
性能向上は測定・主張しない。

## Verification

focused test は request reuse、source mutation、governance configuration mutation、repository
identity mutation、Contract mutation、phase separation、Contract/context mismatch を確認する。
repository preflight、project governance、observer、workspace format、clippy、full workspace test も必要である。

## Remaining risk

他の lifecycle entry point は互換 wrapper を保持し、別途限定された移行までは旧 root-plus-snapshot
形式を使う可能性がある。本 WI は repository 全体の atomic snapshot や cross-request cache を主張しない。

## Merge 後の recovery boundary

PR #735 は hosted checks を通過して `8dcac7ec` として merge された。review 済み PR head は
`06f7d03f` だが、不変の pre-merge finalization receipt は中間 head `251c3867` を記録している。
Runtime はこれを unbound な finalization transition として正しく拒否する。上記の recovery decision
はこの履歴を保持し、WI-758 が新しい merge 後の governance boundary を担当する。
