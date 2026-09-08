# パフォーマンス受入れフィクスチャ

`cargo test -p cockpit-cli --test performance -- --nocapture` を実行すると、warm
な `status` 起動と中規模リポジトリの observe を測定できます。テスト出力にはサンプル数、
中央値、読取ファイル数、observe 経過時間を記録します。

knowledge crate には 10,000 レコードの無関係な依存関係クエリもあり、
`historical records accessed = 0` を検証します。有界検証 receipt は
`nodesPlanned`、`nodesExecuted`、`nodesReused`、`gitCalls`、`filesRead`、
`filesHashed`、`processesSpawned`、`elapsedMs` を記録します。

<50 ms の status と <100 ms の増分 observe はリリース目標であり、証拠のない主張では
ありません。リリースゲートでは対象プラットフォームの実測出力を evidence bundle に添付します。

Runtime は identity-bound な `PerformanceBaseline` を提供します。`runtimeVersion`、
`runtimeDigest`、`repositoryId`、取得時刻、sample、budget が必須です。
旧 schema 1 fixture では `regression_gate.sh <baseline.json> <candidate.json>` を使い、sample 欠落、
zero iteration、identity 不一致、budget regression を拒否します。この gate は取得済み evidence だけを読み、
source fallback を build しません。schema 2 の P0 evidence 契約は `p0_regression_gate.sh` が個別に検査します。
baseline/candidate の Runtime version/digest は異なっても構いませんが、repository identity、repository snapshot、
evidence の完全性、比較可能な環境を結び付けます。budget は
`warm.p50Ms`、`warm.p95Ms`、`warm.p99Ms` のいずれかを明示し、不十分な percentile は gate failure とし、
`elapsedMs` や別 percentile への fallback は行いません。

Verification scheduler は command ごとの resource weight と明示的な resource budget に対応します。
weight が zero または budget 超過なら fail-closed になり、dependency order、protected node、
receipt reuse の意味は変わりません。Repository context と Runtime session は request-scoped であり、
process-level の current repository は作りません。

WI-395 の Rust ネイティブ最適化は、Work Item 集約 status の重複 snapshot を除去し、既存の Git index
読み取り中に source-tree digest を取得し、リモート既定メタデータを 1 回の限定クエリで解決し、observe 中の
再帰的な再ソートも避けます。最適化は request-scoped/identity-bound であり、global repository cache や参照源のインストール手順を導入しません。

Portable `runtime_benchmark.sh <binary> <repo> <output.json> [iterations]
[work-item-id] [budgets.json]` は schema 2 の end-to-end process wall-clock evidence を出力します。取得順を
保持し、最初に測定した独立 CLI process、OS cache warmup 後の独立 CLI process、常駐 MCP の未測定を明示的に
区別します。各 record は raw sample、warmup 数、sample 数、quantile method、Runtime/repository identity、
repository state、data scale、scenario matrix、phase boundary、cache invalidation reason、resource metric を
保持します。Runtime または platform から信頼できる値を得られない read bytes、hash bytes、Git call 数、
child process 数、peak memory は unavailable と記録し、zero にはしません。外部の executable regular file
だけを受け付け、Runtime identity と file SHA-256 を記録して atomic に出力し、source fallback の build/run は
行いません。1 iteration の sanity run から p95/p99 を主張してはなりません。release gate には明示的にレビュー
した budget file と `p0_regression_gate.sh` を使用します。旧 schema 1 fixture では `regression_gate.sh` を使います。

P0 の scenario matrix は、小規模/大量ファイルの clean repository、単一/複数/大規模ファイル変更、大量の
historical Work Item、複数 concurrent validation request、常駐 MCP の repeat query を含みます。1 回の invocation
は指定された repository に束縛され、未選択の scenario は `not_measured` と記録し、結果を合成しません。
取得した事実が形状を証明できる場合だけ measured とします。`small-clean` は clean かつ tracked file 100 件以下、
`many-files-clean` は clean かつ 1,000 件以上、`single-file-change`/`multi-file-change` は変更 path が
1 件/2 件以上、`large-file-change` は 1 MiB 以上の変更 file、`many-historical-wi` は archived Work Item
100 件以上を要求します。Portable harness は concurrent request や常駐 MCP transport を実行しないため、これらは
明示的に `not_measured` のままです。
