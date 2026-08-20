# パフォーマンスベースライン（local evidence）

`cargo test -p cockpit-cli --test performance -- --nocapture` を 2026-08-21 に開発
workspace で実行して取得しました。

| Surface | Fixture | 結果 |
| --- | --- | --- |
| `status` warm startup | 12 samples | 中央値 2 ms |
| repository observation（incremental cache hit） | 200 files、406 files read/hashed | 26 ms |
| knowledge 無関係 query | 10,000 records | historical records accessed 0 |

今回の status 目標（<50 ms）と incremental observation 目標（<100 ms）は達成しました。
初回の uncached scan は別に測定し、受入れ目標は incremental cache-hit path に適用します。
数値はこの machine の evidence であり、普遍的な保証ではありません。
