# ドキュメントマップ

英語文書を機械向け用語の canonical source とします。中国語と日本語の文書は
要約ではなく、意味的に同等でなければなりません。

## ここから開始

- [製品境界](architecture/product-boundary.ja.md)
- [Runtime topology](architecture/runtime-topology.ja.md)
- [バージョニング](architecture/versioning.ja.md)
- [Bootstrap Work Item ルール](work-items/README.ja.md)
- [Repository Protocol v1](protocol/v1/specification.ja.md)
- [パフォーマンス受入れ](../tests/performance/README.ja.md)
- [実測パフォーマンスベースライン](performance/baseline.ja.md)
- [リリースと配布](release/distribution.ja.md)
- [敵対的検証](security/adversarial-validation.ja.md)
- [Work Item ロードマップ](work-items/WI-03.ja.md)

## 開発順序

1. semantics と protocol を凍結する。
2. pure governance core を構築する。
3. repository を一度観測し immutable snapshot を再利用する。
4. verification、lifecycle write、knowledge、attach、MCP を追加する。
5. conformance、性能、adversarial behavior、thin-repository 利用を証明する。

WI-03 から WI-24 に現在の実装状態を記録しています。partial の項目は evidence gate が完了するまで GA ではありません。

Rust runtime が自分自身を governance できるまでは、`docs/work-items` の
Markdown bootstrap ルールを使います。このリポジトリに V1 を install しません。
