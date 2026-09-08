---
author: AI Cockpit maintainers
title: ガバナンス performance budget
description: 必須 verification を弱めない identity-bound な local performance measurement。
audience:
  - contributor
  - maintainer
  - adopter
status: implemented
authority: canonical
lastVerifiedBy: WI-345-reference-governance-cost-batch-15
---

# ガバナンス performance budget

Performance measurement は local engineering evidence であり、必須 check を省略する許可でも hosted provider evidence でもありません。Rust verification crate は typed `PerformanceBaseline`、`PerformanceSample`、`PerformanceBudget`、`PerformanceAssessment` を提供します。baseline には Runtime version/digest、repository identity、capture time、sample、明示的な最大 elapsed-time budget が必要です。

Portable regression gate は baseline と candidate の JSON を読みます。

```sh
tests/performance/regression_gate.sh baseline.json candidate.json
```

missing/zero-iteration sample、invalid identity、budget regression を拒否します。source fallback を build せず、Contract の required verification graph も変更しません。Verification command の resource weight と明示的な resource budget も execution 前に fail-closed です。

## Measurement boundary

Reference の profile P95 report は Rust Runtime authority ではありません。sample が足りないのに established budget を推測せず、timing から governance profile を自動決定せず、local timing を provider/enterprise assurance として表示しません。欠落・stale・identity mismatch は unknown または fail-closed です。

Performance と governance strength は別の次元です。`VerificationTier` と `EvidenceAssurance` は elapsed time、cache hit、worker 数、budget result から導出されません。over-budget report が bottleneck を示しても、protected node と policy-required node は必須のままです。

## Dynamic verification selection

検出された Work Item command は standalone の auto-detected verification と同じ profile-authorized reuse path を使います。repository、snapshot、profile、Runtime、command、scope、stage、runner、base、toolchain、dependency、policy identity がすべて一致した場合だけ reuse します。不一致または impact unknown の場合は宣言された command を実行し、その理由を記録します。reuse の範囲を暗黙に広げたり、必須 check を弱めたりしません。explicit custom command は常に fresh であり、将来 reuse する場合は明示的な custom-command reuse Contract が必要です。

WI-402 は shell と Agent turn をまたいで reusable identity を安定させます。除外するのは session
bookkeeping（`_`、`OLDPWD`、`SHLVL`、mise metadata、`CODEX_*`）だけで、command/toolchain input は
引き続き bind します。content key は source-only であり、Runtime が生成する `.ai/` receipt 自身では
reuse を stale にしませんが、source または非 `.ai` の変更は無効化します。回帰テストは first execution
と exact second-run reuse を確認します。

## Rust ネイティブ最適化の境界

WI-395 は request-scoped status と Work Item 集約 status 投影の重複 snapshot を除去し、
Git index の読み取り中に source-tree digest も取得し、リモート既定ブランチのメタデータを
1 回の限定 Git クエリで解決し、中間結果を並べ替えるためだけの再帰的な再ソートも避けます。snapshot、identity、evidence、
fail-closed の判定は変わりません。参照源のインストール手順はコピーせず、Runtime は
マシン上で共有する外部 binary のままです。各 adopter は明示的な `--repo` と独立した
`.ai/` 状態を使用します。

## 子プロセス待機のブロッキング化 (WI-650)

検証 executor は以前、すべての子プロセスを `child.try_wait()` +
`sleep(10ms)` ループで待機していた。独立したマイクロベンチマークにより、
ほぼ瞬時に終わるコマンドに対して約11msの純粋な待機が追加される一方
（平均12.19ms対1.19ms）、数秒かかるコマンドではブロッキング待機と区別が
つかないことが示された。Unix では `wait_for_child` が子プロセスを専用
スレッドに move し、そのスレッドがブロッキングの `child.wait()` を呼んで
結果をチャンネル経由で報告し、呼び出し側は有界の `recv_timeout` を行う。
タイムアウト・プロセスツリー終了・出力キャプチャの保証は変更していない。
Windows のパスは、本 Work Item では検証していないため、元のポーリング
ループのまま残している。完全な前後比較の証拠は
`docs/work-items/WI-650-verification-wait-blocking.md` を参照。

## Object project への継承

Adopter repository は同じ identity-bound fixture と regression gate を使えますが、repository/Runtime identity はそれぞれ固有です。Shared Runtime は global budget/current project を保存せず、ある repository の timing が別 repository の Work Item を認可することもありません。

Runtime upgrade 後も adopter repository は同じ dynamic rule を継承します。cold verification で receipt を作り、変更のない warm repeat だけがその adopter の repository context 内で reuse できます。adopter acceptance receipt には cold/warm elapsed time、executed/reused nodes、selection reason、Runtime identity、repository identity を記録します。
