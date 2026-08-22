---
author: AI Cockpit maintainers
workItemId: WI-135-repository-bound-evidence
title: Repository に束縛された retention と close evidence
description: すべての lifecycle 境界で retention metadata と close receipt を現在の repository と Work Item に束縛する。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-135-repository-bound-evidence
---

# WI-135 — Repository に束縛された retention と close evidence

## Intent

コピーされたもの、壊れたもの、別 repository の retention policy や close receipt が、
現在の repository の治理事実として受理されることを防ぐ。

## 境界

- retention policy の利用前に schema version、repository identity、Work Item identity、
  timestamp、retention 値を検証する。
- verification evidence に埋め込まれた retention と repository-local policy が両方ある
  場合は一致しなければならない。
- close receipt は repository identity を書き込み、必須とする。欠落または foreign の
  receipt は archived Work Item を `closed` に昇格させず、有効な human decision として表示しない。
- 過去の evidence bytes は不変であり、この WI は書き換えない。

## 受入れ

- 有効な retention と close record は引き続き読み取れる。
- foreign、欠落、形式不正、未知フィールド、schema 不一致、repository 横断 record は
  Outcome、MCP、finish、archive、close、status、purge の全経路で fail closed する。
- repository/Work Item binding と legacy historical projection の回帰テストを追加する。

## 検証

Archived verification evidence は `.ai/evidence/WI-135-repository-bound-evidence.verification.json`、
close decision は `.ai/decisions/WI-135-repository-bound-evidence.close.json` である。この WI は
Task Report や Recovery state 機能を導入しない。
