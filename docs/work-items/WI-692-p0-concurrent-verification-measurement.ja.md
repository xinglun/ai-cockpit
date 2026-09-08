---
author: AI Cockpit maintainers
title: "WI-692 — P0 並行 verification 測定"
description: "PhysicalSingleFlightCoordinator の統合を検討する前に、実際の並行 verification request path を測定します。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-692-p0-concurrent-verification-measurement
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-692-p0-concurrent-verification-measurement
---

[English](WI-692-p0-concurrent-verification-measurement.md) · [简体中文](WI-692-p0-concurrent-verification-measurement.zh-CN.md)

# WI-692 — P0 並行 verification 測定

## Intent

同じ repository と command に対する並行 verification request が物理実行を重複させるかを測定し、
後続の `PhysicalSingleFlightCoordinator` 統合が妥当かを判断します。North Star は
Calibrated Human-Agent Trust です。evidence、repository isolation、authorization、fail-closed
behavior、recovery を、推測された throughput gain より優先します。

## Scope and validity boundary

この Work Item は再現可能な外部測定 harness と evidence のみを追加します。harness は公開された
`ai-cockpit verify` CLI を、clean fixture に対して独立 process として起動し、request order、raw result、
exit code、command identity material、receipt metrics、round wall time、environment identity を保存します。
coordinator を直接呼ばず、`crates/**`、verification semantics、gate、production caller は変更しません。

したがって、観測された 1 round あたり 4 回の physical execution は independent CLI process の baseline です。
process 内 MCP/service request path が安全に 1 回へ共有できることの証明ではありません。後続統合は、実際の
path で同一 identity の重複と material cost を示し、かつ request ごとの authorization、evidence binding、
failure、cancellation、resource boundary を維持できる場合に限ります。

## Bottleneck evidence and hypothesis

仮説は、同一 identity の並行 request が physical command を繰り返し、single-flight の測定可能な対象に
なる可能性です。測定は release Runtime `0.2.87`、file digest
`sha256:7610b70b38dca6520ee8ee8cc15b838bb3c1d5d7f4b350234b02f055334fb3a6`、
`99f7d2323ffb59b1e3edd6c1c833b00f50fb9698` の clean detached fixture で行いました。
fixture は 9,955 tracked files、working tree clean、期待された repository identity です。

この revision の production call graph には coordinator の definition と implementation はありますが、
production caller は 0 件でした。参照は `crates/cockpit-verification/tests/physical_execution.rs` に限られます。
従って independent CLI process の重複は実測ですが、現在 production から未使用の in-process coordinator を安全に
接続する根拠にはなりません。

## Measurement Contract and raw result

harness は raw sample を round order と request index order のまま保持し、nearest-rank quantile の計算時だけ
copy を sort します。sample count、raw samples、warmup/identity-probe order、Runtime/repository identity、
command identity、process count、unavailable metric の理由を記録します。測定前に Runtime identity を取得したため、
true cold cache は主張せず `coldCacheClaim` は `false` です。

初回実行では、governance receipt が `passed: false` でも `returnCode == 0` だけで success と分類する harness defect
が見つかりました。誤った raw run は
`.ai/evidence/external/WI-692-p0-concurrent-verification-measurement.raw-initial-measurement-bug.json` に保存し、
exit code 0 と `governancePassed == true` の両方を要求するよう修正して final measurement を再実行しました。初回記録を
黙って置換してはいません。

最終 round wall-clock 結果は次のとおりです。

| Scenario | Rounds × concurrency | Successful / failed requests | Physical executions / round | p50 / p95 round wall ms |
| --- | ---: | ---: | ---: | ---: |
| Same identity | 20 × 4 | 80 / 0 | every round 4 | 720.006 / 1094.730 |
| Distinct command identity | 20 × 4 | 80 / 0 | every round 4 | 503.963 / 726.116 |
| Failure propagation | 20 × 4 | 0 / 80 | every round 4 | 473.366 / 964.666 |
| Resource contention | 20 × 4 | 80 / 0 | every round 4 | 674.100 / 993.346 |
| Cancellation | 1 × 1 | 0 / 1 | unavailable: no governance receipt | 106.182 / unreliable |

20 round の p50/p95 は宣言した reliability floor を満たしますが、p99 は 100 samples 未満なので reliable claim ではありません。
cancellation は 1 sample だけなので、reliable percentile は主張しません。全 raw values と derived summary は
`WI-692-p0-concurrent-verification-measurement.*.json` に保存されています。

queue wait、CPU time、peak memory、read bytes、hashed bytes は公開 CLI または platform collector から取得できず、
unavailable と理由を記録しました。0 には置換していません。観測 request concurrency と receipt process count は
記録しましたが、nested Cargo resource、resident MCP latency、process 内 service queue は未測定です。

## Decision

今は **coordinator 統合を見送ります**。4 つの independent CLI process が各 round で 4 回実行したことは確認できましたが、
production caller が存在せず、現 path は安全な共有に必要な identity/evidence boundary を示せません。successor の前提は次のとおりです。

1. 実際の in-process MCP または service path で、同じ repository、Work Item、command、Runtime、toolchain identity を測定する。
2. physical result を共有する前に各 waiter の authorization と evidence receipt を bind する。
3. その path で failure、timeout、cancellation、waiter exit、nested Cargo resource を測定する。

## Correctness, isolation, and governance

focused test は sample order の保持、unavailable metric を 0 にしないこと、command identity argument の保持を確認します。
call-graph evidence は definition、test reference、production caller を区別します。測定は repository-bound、external executable、
clean fixture であり、Rust production code は変更していません。既存の governance gate、authorization、evidence semantics、
recovery behavior は弱めていません。

Runtime verification、review 済み PR delivery、archive、close、documentation promotion が終わるまで、この Work Item は
`in_progress` です。production performance benefit は主張せず、見送り判断と unknown は measurement summary に bind されています。
