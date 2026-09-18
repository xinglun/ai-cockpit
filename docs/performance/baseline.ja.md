---
author: AI Cockpit maintainers
title: "パフォーマンスベースライン"
description: "再現可能な local performance evidence と release 上の制限。"
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - performance_baseline
---

# パフォーマンスベースライン（local evidence）

この baseline は次の条件で取得しました。

```text
command: cargo test -p cockpit-cli --test performance -- --nocapture
source base: 9177b119d3232bbc48dacca71c0beff31089e82b
host: aarch64-apple-darwin（Darwin arm64）
toolchain: rustc/cargo 1.94.1
profile: dev、incremental test fixture
date: 2026-08-21
```

測定時の source tree は未 commit の local candidate でした。数値は machine-specific な
baseline であり release evidence ではありません。公開前に immutable release candidate
から再測定してください。

| Surface | Fixture | 結果 |
| --- | --- | --- |
| `status` warm startup | 12 samples | 中央値 23 ms |
| repository observation（incremental cache hit） | 200 files、405 files read | 63 ms |
| knowledge 無関係 query | 10,000 records | historical records accessed 0 |

今回の status 目標（<50 ms）と incremental observation 目標（<100 ms）は達成しました。
初回の uncached scan は別に測定し、受入れ目標は incremental cache-hit path に適用します。
raw command output は release candidate の acceptance record と一緒に保持してください。

## 過去の object repository capture（WI-889）

WI-889 は `0.2.93` baseline と同じ `aarch64-apple-darwin`、Rust/Cargo
`1.98.1` 上で、現行 `0.2.95` の paired capture を取得した。goods-garden、
sentinel、ai-investigation-orchestrator は一時 view だけから観測した。七つの
scenario で 38 operation comparison を行い、各 operation は 100 個の valid
warm sample を持つ。5 ms noise budget の p50/p95 判定はすべて `within_noise`
で、p99 も tail diagnostic として保持する。これは比較可能な現行 evidence
を確立するが、速度改善の証明ではない。提供された object repository はすべて
harness の `<=100` tracked-file 閾値を超えたため small-clean は unavailable
である。完全な raw capture と counter は WI-889 evidence archive に保持する。

## 過去の paired capture（WI-876）

現在の候補は同じ `aarch64-apple-darwin` machine、`rustc/cargo 1.98.1`、Runtime `0.2.93` で、外部 baseline binary と個別に build した candidate binary を paired 測定しました。小規模 clean、many-file clean（ORG-X）、大量 historical Work Item（ai-investigation-orchestrator）、single-file change、multi-file change、large-file change、evidence path change の七つの isolated fixture を使い、各 operation は 100 個の有効な warm sample を持ちます。raw sample、identity、counter、unavailable reason は `.ai/evidence/WI-876-performance-current-proof/paired-current-seven-scenarios.json` に保持します。

5 ms の noise budget で比較した結果、改善は**まだ証明できません**。23 operation は noise 内、2 operation は改善、10 operation は暫定 noise budget を超えました。これは測定 evidence であり release performance pass ではありません。diagnostics on/off overhead は `diagnostics-overhead-org-x.json` に分離しています。開発 cycle cost は別報告で、現在取得できたのは Contract→checkpoint の 63,000 ms（1 sample）のみです。agent operation/preflight reject 数、verification→finish、merge 後 cleanup は明示的に unavailable です。

## 現行 v0.2.98 supplemental capture（WI-905）

WI-905 は WI-876 と WI-889 の過去 evidence bytes を書き換えず、公開版の
identity を補足した。同じ `aarch64-apple-darwin`、Rust/Cargo `1.98.1` 上で
公開 `v0.2.93` と `v0.2.98` binary を paired 測定し、各比較 operation に
100 個の valid warm sample を持たせた。clean な object view は goods-garden
（396 tracked files）と ORG-X（1,240 tracked files）で、どちらの main branch
も変更・merge していない。正確な環境 identity、raw capture、checksum は
`.ai/evidence/WI-905-performance-v098-evidence/raw/` にある。

最初の 10 operation 比較は既存の 5 ms noise budget で 3 つの暫定 regression
flag を出した。その後の独立した 100-sample observe repeat では observe flag
を再現しなかった（goods-garden: p50 -2.397 ms / p95 +2.148 ms、ORG-X:
p50 -0.240 ms / p95 +1.314 ms）。inspect tail は再測定しておらず、未解決の
diagnostic として残る。internal `git_snapshot` p95 は goods-garden で
30.687 ms から 24.813 ms、ORG-X で 34.796 ms から 31.576 ms に改善したが、
end-to-end CLI の利用者向け速度改善はまだ証明していない。candidate の
`work-item-outcome --delivery --json` は 100 valid sample で p50 241.329 ms、
p95 264.831 ms、p99 371.828 ms。diagnostic on/off overhead は分離して報告し、
Runtime が公開しない cache invalidation event は unknown のまま保持した。
取得済み Contract→reviewable PR 以外の開発 cycle stage は unavailable であり、
0 として扱っていない。
