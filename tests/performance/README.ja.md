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
`regression_gate.sh <baseline.json> <candidate.json>` は sample 欠落、zero iteration、
identity 不一致、budget regression を拒否します。この gate は取得済み evidence だけを読み、
source fallback を build しません。

Verification scheduler は command ごとの resource weight と明示的な resource budget に対応します。
weight が zero または budget 超過なら fail-closed になり、dependency order、protected node、
receipt reuse の意味は変わりません。Repository context と Runtime session は request-scoped であり、
process-level の current repository は作りません。

WI-395 の Rust ネイティブ最適化は、Work Item 集約 status の重複 snapshot を除去し、既存の Git index
読み取り中に source-tree digest を取得し、リモート既定メタデータを 1 回の限定クエリで解決し、observe 中の
再帰的な再ソートも避けます。最適化は request-scoped/identity-bound であり、global repository cache や参照源のインストール手順を導入しません。
