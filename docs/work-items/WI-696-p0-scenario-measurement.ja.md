---
author: AI Cockpit maintainers
title: "WI-696 — P0 シナリオ計測"
description: "既存の P0 ベンチマークで実シナリオ行列を計測し、ボトルネック順序と有界な後続予算を証拠化する。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-696-p0-scenario-measurement
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-696-p0-scenario-measurement
---

[English](WI-696-p0-scenario-measurement.md) · [简体中文](WI-696-p0-scenario-measurement.zh-CN.md)

# WI-696 — P0 シナリオ計測

## 目的と境界

本 Work Item は、Runtime のガバナンス動作、認可、リポジトリ隔離、復旧意味論を変更せず、既存の schema 2 ベンチマークを実シナリオで実行する。North Star は Calibrated Human-Agent Trust のままである。開発 Runtime は `origin/main` の revision `513e7e72523097893c658ec76c25706fd5b266b8` から構築した `0.2.87` で、ファイル digest は `sha256:faacc6a3368f56bc9274264c111500157fb79354260dba60898a3351b5c3e4dd` である。公開リリースの受入れは別の境界に従う。

各シナリオは、identity probe 後の初回計測、OS キャッシュを温める 1 プロセス、20 個の独立 warm プロセス、取得順の raw sample、nearest-rank、Runtime/リポジトリ identity、snapshot、環境、データ規模、利用不能理由を保存する。初回を真の cold cache と呼ばず、20 サンプルでは p95、100 未満では p99 を信頼できる値としない。

## 結果と予算

最終 evidence は `.ai/evidence/external/WI-696-p0-scenario-measurement.*.budgeted2.dev-faacc6a3.json`、集計は `.ai/evidence/external/WI-696-p0-scenario-measurement-summary.json` にある。

| シナリオ | status warm p50/p95 ms | observe warm p50/p95 ms |
| --- | ---: | ---: |
| small-clean | 82.696 / 85.517 | 59.106 / 60.704 |
| single-file-change | 93.616 / 103.280 | 71.838 / 72.722 |
| multi-file-change | 95.430 / 99.833 | 72.077 / 73.432 |
| large-file-change | 99.895 / 101.456 | 75.675 / 77.739 |
| many-files-clean | 3,095.188 / 3,126.662 | 144.646 / 148.613 |
| many-historical-wi | 3,103.234 / 3,588.843 | 145.194 / 148.943 |

大規模リポジトリでは `status` が明確な現在のボトルネックである。これは計測上の順序であり、未承認の実装変更を意味しない。Runtime 内部の Git 呼び出し、read/hash bytes、子プロセス、peak memory、phase timing、cache invalidation は利用不能理由付きで保存し、ゼロにはしない。Darwin の filesystem type は comparator の信頼できる環境キーとして採用されなかったため、このホストでの異なる Runtime の比較は fail closed となる。

並行の独立 CLI 計測では、同一 identity・20 rounds・concurrency 4 で毎 round 4 physical executions、round p50/p95 は 441.652/454.629 ms だった。現在の `origin/main` には `PhysicalSingleFlightCoordinator` の production caller がなく、統合は当面見送る。resident MCP は transport がなく `not_measured` とする。

予算ファイルは `.ai/evidence/external/WI-696-p0-scenario-measurement-budgets/` にある。これは反復した開発計測と有界なノイズ余裕から作った regression ceiling であり、性能向上の主張ではない。後続 WI は target path と paired baseline/candidate の改善閾値を先に Contract 化し、非対象シナリオの予算を守り、完全で比較可能な環境 evidence による `p0_regression_gate.sh` を通す必要がある。本 Work Item は実装最適化を行わない。

## 外部検証の制限

リポジトリ全体の documentation acceptance、status consistency、
`closed-work-item --check-all` は、別 Work Item の WI-694 ドキュメント昇格が
まだ `in_progress` のため fail closed している。正確な出力は
`.ai/evidence/external/WI-696-p0-scenario-measurement.validation-limitation.json`
に保存した。この Work Item は WI-694 のファイル、他 agent の worktree、branch、PR、
evidence を変更しない。WI-694 完了後にこれらの検証を再実行する。
