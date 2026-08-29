---
author: AI Cockpit maintainers
title: "Work Item スタイルガイド"
description: "レビュー可能で evidence-bound な Work Item を書くための実践ガイド。"
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: WI-390-reference-style-guide
capabilityClaims:
  - work_item_style_guidance
---

# Work Item スタイルガイド

[English](work-item-style-guide.md) · [简体中文](work-item-style-guide.zh-CN.md)

このガイドは、人がレビューでき、インストール済み Rust Runtime が検証できる Work Item の
書き方を説明します。第二の Contract schema ではありません。Contract は intent、authority、
scope、acceptance、required evidence を人が所有する source のまま保持します。

## 先に結果を書く

実装方法より先に、完了時に何が真であるべきかを書きます。問題の背景や user benefit が提示
されていない場合は、その事実を明示します。ファイル名、検出された技術、Agent の prose から
動機、影響、承認、完了を推測してはいけません。

現在の Contract field は意図的に使います。

- `intent` と `goal` は、人が所有する目的と期待結果を表します。
- structured intent には `businessGoal`、`userGoal`、`problem`、`constraints`、`nonGoals`、
  `rationale` を記録できます。すべて任意で、owner が提供していなければ unknown のままにします。
- `intentAlignment` は実装後の任意の Summary projection です。問題、制約、non-goal、rationale が
  実際に扱われたかを記録し、元の intent は書き換えません。

## 問題と境界を先に定義する

背景が分かっている場合だけ Work Item の理由を説明します。編集前に repository-relative な
`scope` と `outOfScope` を宣言します。scope は認可境界であり、事後の changed files 一覧では
ありません。明示的な non-goal により、レビューで意図しない拡張を検出できます。

## 検証可能な acceptance にする

Acceptance criteria は、人または宣言された verification command で確認できる必要があります。
「Contract validator が通る」「文書の route link が解決する」のような文を優先し、「よさそう」
のような主観的な表現は避けます。`A<n>:` の番号付き criteria は Summary evidence に bind
できます。番号なし criteria は読みやすい source-language declaration として残ります。Runtime
は criteria や evidence mapping を推測しません。

## Governance decision は人が所有する

`authority`、承認、risk acceptance、unknown がある状態で継続する決定は、責任を持つ人または
明示的に委任された provider のものです。Runtime は shape、identity、freshness、evidence を検証
しますが、欠落 field を許可に変えません。yellow/red の preflight はレビュー境界であり、編集や
finish の許可ではありません。

## 必要十分なプロセスを使う

既存の lifecycle と verification capability を使います。レビューまたは audit value を保つ場合
だけ field、gate、承認手順を追加します。repository に応じた Light/Standard/Strict profile を選び、
強い Verification Tier を強い Evidence Assurance と同一視しません。

## 実行可能な verification を記録する

repository で実行できる check を宣言し、インストール済み Runtime と明示的な `--repo` で実行します。
Verification receipt は Work Item、repository snapshot、Runtime identity に bind されます。宣言だけ
では evidence にならず、存在するだけの path も pass にはなりません。

## 新しい概念の前に既存概念を拡張する

新しい概念を追加する前に、現行の Contract、Summary、scenario、evidence、decision、policy field を
確認します。まず review model を文書化し、決定的な machine check が必要な場合だけ schema を追加します。
source language と governance bytes を保ち、presentation localization で意味を変更しません。

## Adopter project への継承

adopter repository は repository-local `.ai/` と Agent adapter を通じて同じ読者向けルールを継承し、
shared Runtime は project 外部に残ります。`--repo` ごとに repository identity、Contract、evidence、
knowledge は分離されます。本ページは reference installer の command や Runtime implementation を
コピーせず、Rust-native interface に適用可能な governance semantics だけを引き継ぎます。
