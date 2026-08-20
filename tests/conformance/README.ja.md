# Conformance harness

各 case には repository material、contract、evidence、明示的な governance input、期待する
semantic result があります。Rust test はファイルを読み、format ではなく decision field を比較します。
`v1-reference.lock` は corpus 作成に使った V1 reference commit を記録し、通常の build は V1 を取得・実行しません。
