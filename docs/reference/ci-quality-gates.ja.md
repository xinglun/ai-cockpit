---
author: AI Cockpit maintainers
title: CI Contract 対応品質ゲート
description: 動的な CI ルートと Rust-native Contract ゲート。
audience:
  - contributor
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-291-ci-contract-aware-gates
---

# CI Contract 対応品質ゲート

AI Cockpit はリソースを意識した品質ルートを使用します。すべての変更で
最も高価な検査を無条件に実行する設計ではありません。

- `light`：ドキュメントだけの変更に対する集中検査
- `standard`：ソース、テスト、workspace 検査を追加
- `strict`：ガバナンス、workflow、release-owned、高リスク、未知の表面で必須

ステージ下限とリスクのエスカレーションは profile を引き上げます。要求された
profile は自動判定を引き上げることだけができ、下げることはできません。
canonical manifest はコマンド一覧を定義し、route receipt は manifest、Git の
base/head、変更パス、Contract のパス/digest、有順序の gate ID を束ねます。

## Rust の権威と Python shadow

active Contract がある standard/strict の Pull Request では、CI はリポジトリの
コマンドを実行する前に read-only Rust gate を実行します。

```text
Python route/manifest plan
        ↓
Rust Contract gate（権威、.ai を変更しない）
        ↓
Python gate runner と Cargo/static checks（shadow 比較）
```

Rust gate は regular Contract、repository identity、base revision、現在の
snapshot、typed Contract invariant、policy に束ねられた
intent/scenario/operation/stage route、Agent-Risk/preflight 投影を検証します。
安定した receipt digest、decision state、verification tier、evidence assurance
を含む `repository_contract_quality_gate` JSON を出力します。黄色または赤の
結果は非ゼロで終了し、リポジトリコマンドを認可しません。

収束期間中は Python route と runner を残します。hosted shadow 比較で意味の一致を
確認した後の別 batch でのみ、重複した policy を削除します。この gate は参照源の
全 workflow matrix、依存 planner、release-preflight 順序を実装するものではありません。

## ソース中心のスナップショット識別子

リポジトリ・スナップショットの識別子はソース中心である。追跡対象のソースツリーと
`.ai` 以外のワークツリー事実を束縛し、Git の `HEAD`、絶対パス、ガバナンス専用の
`.ai/` コミットは除外する。これにより、検証後に Contract、Summary、Outcome の記録を
通常どおりコミットしても証拠は stale にならず、ソース変更による古い証拠の再利用は
引き続き拒否される。

## Evidence と Release の境界

CI gate は reviewed change に対する source build の検査です。診断用に Runtime
identity を記録しますが、公開 Release artifact ではありません。Release と adopter
acceptance は、immutable な downloaded tag、archive/binary checksum、SBOM/provenance、
公開 artifact harness を引き続き要求します。
CI gate は `.ai/` の Contract、Summary、checkpoint、verification、decision を書き込みません。
これらの可変記録の権威は lifecycle command にあります。

object project でも同じ境界を守ります。共有 Runtime は工程の外部にあり、すべての
request は明示的な `--repo` を持ち、repository Evidence は分離されます。
