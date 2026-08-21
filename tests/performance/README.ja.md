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
