---
author: AI Cockpit maintainers
title: "WI-715 — P1 大規模履歴 status 候補の判断"
description: "大規模履歴 status 経路のリクエスト内重複除去候補を測定し、受入契約を満たさない場合は証拠付きで見送る。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-715-p1-large-history-status
lastVerifiedBy: WI-715-p1-large-history-status
---

[English](WI-715-p1-large-history-status.md) · [简体中文](WI-715-p1-large-history-status.zh-CN.md)

# WI-715 — P1 大規模履歴 status 候補の判断

## 目的

1 回の不変な `status` 観測内で検証済み close receipt を再利用すると、
`many-historical-wi` 経路の重複した読み取り・解析を減らせるか測定する。同時に
Calibrated Human-Agent Trust、証拠の有効性、リポジトリ分離、認可、復旧動作を維持する。

## 範囲と受入契約

対象はリクエスト内の status 観測、焦点テスト、外部ベンチマーク証拠、三言語の記録のみ。
永続キャッシュ、リポジトリまたは WI をまたぐ再利用、coordinator、IncrementalMerkle
再利用、大ファイルのストリーミング、待機方式の置換、P3 アーキテクチャ変更は含めない。
候補は事前登録した warm status p50 と p95 の各 5%以上改善、非対象予算、挙動一致を要求した。

## 測定と結果

ペアの開発用バイナリは warm サンプル 20 件、OS キャッシュのウォームアップ 1 回で測定した。
最初の測定は identity probe の後なので、真の cold cache とは呼ばない。p99、常駐 MCP、段階別時間、
Runtime 内部 Git/I/O カウンタ、ピークメモリは、ハーネスまたはホストが確実に取得できない場合に
不可用として記録し、ゼロにはしていない。このホストでは信頼できるファイルシステム比較キーを
取得できず、回帰ゲートは fail closed した。

| 順序 | baseline status warm p50/p95 | candidate status warm p50/p95 | candidate 差分 | 判断 |
| --- | ---: | ---: | ---: | --- |
| baseline → candidate | 3483.095 / 4681.167 ms | 3334.552 / 3478.871 ms | -4.26% / -25.68% | 閾値未達、ゲート fail closed |
| candidate → baseline | 3315.139 / 4260.963 ms | 3435.055 / 4453.557 ms | +3.62% / +4.52% | 閾値未達、ゲート fail closed |

予算に紐付けない追加実行は候補に不利（p50 +1.35%、p95 +9.28%）で、補助証拠としてのみ保存した。
候補コードとテストは削除し、製品コードの性能変更はマージしていない。

## 正しさと証拠

clean、changed、malformed-close fixture で `inspect`、`status`、`doctor`、`observe` を比較し、
12 比較すべてで baseline/candidate の結果が一致した。終了コード差分は 0、runtime digest フィールド
だけを除く正規化出力差分も 0 だった。生データと派生記録は
`.ai/evidence/external/WI-715-p1-large-history-status.*` に保存し、ゲート結果と実験サマリーを判断記録とする。

## 判断と次の行動

判断：`declined`。この WI はレイテンシ、CPU、I/O、メモリ、常駐 MCP の改善を主張しない。信頼できる
環境比較キーと、実際のボトルネックを示す強い段階別測定が揃い、複雑性を正当化できるまでは、この
マイクロ最適化を受入済み変更として再試行しない。次の最適化 WI は最新のレビュー済みデフォルト分岐から
開始し、今回の見送りを不変証拠として保持する。
