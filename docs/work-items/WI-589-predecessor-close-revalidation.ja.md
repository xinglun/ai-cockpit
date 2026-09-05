---
author: AI Cockpit maintainers
title: "WI-589 — Contract amendment 再検証後の predecessor close"
description: "successor が修訂 Contract を再検証した後、旧 provider finalization receipt を歴史証拠として close に束縛する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-589-predecessor-close-revalidation
lastVerifiedBy: WI-589-predecessor-close-revalidation
terminalArchive: .ai/work-items/archive/WI-589-predecessor-close-revalidation.contract.json
terminalVerification: .ai/evidence/WI-589-predecessor-close-revalidation.verification.json
terminalFinalization: .ai/decisions/WI-589-predecessor-close-revalidation.finalize.046eea80433b45884c522474d2bca7da061b2056187418e638d962d86699db3d.json
terminalDecision: .ai/decisions/WI-589-predecessor-close-revalidation.close.json
---

[English](WI-589-predecessor-close-revalidation.md) · [简体中文](WI-589-predecessor-close-revalidation.zh-CN.md)

# WI-589 — Contract amendment 再検証後の predecessor close

## 目的

レビュー済み Contract の修訂を terminal successor が再検証した後、archive 済み predecessor を正直に close できるようにします。旧 Provider finalization receipt は歴史証拠として扱い、bytes、path、digest、sequence を保持し、`direct_merge_no_pr` へ再分類しません。

## 境界

この互換経路は狭く append-only です。successor が現在の Runtime による verification、Provider finalization、明示的な human close を完了した場合に限り、predecessor close は正確な旧 finalization head を `historical_low` として再検証に束縛できます。missing、malformed、foreign、stale、contradictory な lineage は fail-closed のままです。direct-merge schema、adopter script、対象 repository、reference source 実装は対象外です。

## 受入条件

1. 旧 Runtime の PR receipt は、Contract amendment successor が terminal かつ repository-bound の場合だけ historical として投影できます。
2. predecessor close は正確な finalization path、digest、sequence を記録し、元の receipt bytes を保持します。
3. close record は successor の current revalidation と provider の historical evidence を区別し、PR や direct-merge 分類を捏造しません。
4. 未完了または改ざんされた successor、archive、Contract、evidence、receipt binding は fail-closed となり、部分的な close record を生成しません。

## 検証

recovery 回帰テストと locked workspace 全体テストを実行します。三言語の command reference に対応する recovery path と fail-closed 境界を記載します。
