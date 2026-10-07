---
author: AI Cockpit maintainers
title: "協作場景行列"
description: "状態と遷移から生成した協作言語の場景行列。実際に観測した事例、既存文書に基づく事例、人工的に構成した事例を明確に区別する。"
audience: [adopter, contributor, maintainer, reviewer]
status: current
authority: canonical
lastVerifiedBy: WI-781-trust-diagnostics
---

# 協作場景行列

本ページは [`collaboration-scenario-matrix.json`](collaboration-scenario-matrix.json)
の読みやすい対訳であり、自動チェックが消費する構造化された事実の出典は JSON
側である。本ページは現在の
[協作言語契約](collaboration-language-contract.ja.md)を拡張し、本リポジトリが
サポートするライフサイクル、証拠、授権、composition 実行、操作の意味に基づいて
具体的な場景を記述する。

## 本行列の読み方

各場景は以下を明示する:

- **出典種別(sourceType)**——`observed`(2026-09-08、WI-679/WI-680 の納品中
  に本リポジトリで実際に実行し、正確なコマンド/出力を引用したもの)、
  `documented`(既存の権威ある文書を言い換えたもので、新たな実行は伴わな
  い)、`designed`(必須カテゴリのうち実データが存在しない箇所を埋めるため
  に構成した合成事例で、実際の使用効果が証明されたと主張するものではな
  い)。
- **カテゴリ**——ライフサイクル遷移、証拠、授権、検証、合併/クローズ、歴史
  照会、Agent/セッションの引き継ぎ、多言語/多入口の一致性、composition の実行/
  retry、プロセス監督、旧版 composition 記録。
- **期待される結果**と、それが検証する**意味論的不変量**(協作言語契約の
  十の不変量を参照)。

本専項自身のスコープ方針に従い、本行列はすべての状態とすべての操作の完全な
直積を網羅するのではなく、重要な境界と混同しやすい組み合わせを優先する。

## カバレッジ概要

| カテゴリ | 場景 | 実観測 | 既存文書 | 人工構成 |
| --- | --- | --- | --- | --- |
| ライフサイクル遷移 | SCN-001..005 | 4 | 0 | 0(SCN-005 は統治上の拒否ではなく運用上の落とし穴) |
| 証拠 | SCN-006..008 | 1 | 2 | 0 |
| 授権 | SCN-009..012 | 1 | 3 | 0 |
| 検証 | SCN-013..015、SCN-025 | 3 | 1 | 0 |
| 合併/クローズ | SCN-016..019 | 3 | 1 | 0 |
| 歴史照会 | SCN-020 | 0 | 1 | 0 |
| Agent/セッション引き継ぎ | SCN-021..022、SCN-026 | 2 | 1 | 0 |
| 多言語/多入口 | SCN-023..024 | 0 | 2 | 0 |
| 最終化の観測 | SCN-027..033 | 6 | 0 | 0 |
| composition 実行 | SCN-034 | 0 | 1 | 0 |
| composition retry | SCN-035..036 | 0 | 2 | 0 |
| プロセス監督 | SCN-037 | 0 | 1 | 0 |
| 旧版 composition 記録 | SCN-038 | 0 | 1 | 0 |
| 検証 retry の境界 | SCN-039 | 0 | 1 | 0 |

SCN-033 は `unavailable` として別途記録し、上表の実観測・既存文書・人工構成の件数には含めない。

SCN-034 から SCN-039 は Contract A12-A14 と現在の実装に基づく `documented`
場景であり、このクラウド実行で観測済みという主張ではない。新しい `designed`
場景は追加していない。

## 完全な行列

完全な構造化場景表(SCN-001 から SCN-039;番号は安定しており、将来の
Work Item は拡張のみ可能で番号の振り直しは行わない)は
`collaboration-scenario-matrix.json` を参照。以下は代表的な抜粋:

| ID | カテゴリ | タイトル | 出典 | 結果 |
| --- | --- | --- | --- | --- |
| SCN-003 | ライフサイクル遷移 | 前身に有効なクローズ決定がない間、新規 WI の起票が拒否される | 実観測 | 拒否 |
| SCN-006 | 証拠 | 検証後の Contract 変更で証拠が無効化され `finish` が拒否される | 実観測 | 拒否 |
| SCN-013 | 検証 | `acceptanceEvidence`/`intentAlignment` が無いと `finish` が阻断される | 実観測 | 拒否 |
| SCN-014 | 検証 | `finish` が緑で成功しつつ、明示的に合併を授権しない | 実観測 | 受理 |
| SCN-017 | 合併/クローズ | 合併確認がプラットフォーム(Runtime ではない)の権限境界で拒否される | 実観測 | 拒否 |
| SCN-019 | 合併/クローズ | 合併前に発見された文書上の欠落は、履歴の書き換えではなく後続 Work Item で修復される | 実観測 | 後続WIへ延期 |
| SCN-022 | Agent/セッション引き継ぎ | ある Agent の未完了の前身クローズが、別の Agent の新規起票を構造的に阻止する | 実観測 | 前身クローズまで拒否 |
| SCN-025 | 検証 | 表示された選択肢、選択されたテストデータの決定、中断、再開後の Runtime 遷移が一致する | 実観測 | 安全に再試行可能な一致 |
| SCN-026 | Agent/セッション引き継ぎ | 新しいサブプロセスが会話履歴なしで Runtime 記録から引き継ぎを再構築する | 実観測 | 明示的な阻断を伴う状態再構築 |
| SCN-034 | composition 実行 | 実行は合格しても cleanup が deferred なら unknown で、再利用できない | 既存文書 | 非終端 unknown |
| SCN-036 | composition retry | binding 済み Linux system no-op のみ deferred-cleanup fresh retry が可能 | 既存文書 | 新 attempt、旧 tree を保持 |
| SCN-037 | プロセス監督 | Unix/Windows process-group backend は Linux ECHILD を主張しない | 既存文書 | backend ごとに異なる |
| SCN-038 | 旧版 composition | legacy v1/v2 boolean は再利用可能な v3 証拠にならない | 既存文書 | 互換表示のみ |
| SCN-039 | 検証 | Cargo の初回実行は deferred-cleanup retry のサポートを意味しない | 既存文書 | spawn 前に retry を阻止 |

## 既知の限界

本行列は引き続き手作業で整理された意味論的な索引であり、完全な自動テスト
スイートではない。JSON の出典には実行可能なチェック登録表を追加し、各対象
場景の入力事実、期待する意味、テスト入口を結び付けた。WI-781 は人間向け
レポートの意味論的一致、finalization アクションの分類、有界 Outcome 組立て
再試行、段階別 Runtime 診断を追加し、未対応のプロセス数は unavailable のまま
扱う。すべての不変量や状態の組合せを完全な直積でカバーすると主張するもの
ではなく、将来の Work Item は既存の番号を振り直さずに有界なチェックを追加
できる。

追加された composition 場景は文書上の境界であり、hosted acceptance receipt
ではない。Cargo コマンドは admission 済みの初回実行として実行できるが、現在の
deferred-cleanup retry 方針が認めるのは保護された system no-op `true` 一つだけで、
Cargo や任意のテストの retry をサポートするものではない。projection は Work Item
の `finish` gate を置き換えない。
