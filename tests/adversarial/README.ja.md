# 敵対的検証サーフェス

v2 conformance corpus は 15 個の semantic case を含み、各言語（英語、日本語、中国語）で各 case に
5 件の wording variant があります。crate の統合テストは、すべての variant が同じ canonical governance
decision になることを要求します。Manifest は RAI-01 から RAI-12 の status も bind するため、
`not_proven` と `partial` の境界を pass と取り違えません。

conformance corpus と crate の統合テストは、スコープ逸脱、破壊的権限、欠落/古い/矛盾した
証拠、未対応の完了宣言、リポジトリ prompt injection、悪意ある削除、Work Item 間証拠、
未知 provider、テスト/カバレッジ弱体化、archive 復旧、MCP パス境界、検証 cwd 境界を検証します。
