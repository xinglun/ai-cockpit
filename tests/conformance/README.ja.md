# Conformance harness

各 case には repository material、contract、evidence、明示的な governance input、期待する
semantic result があります。通常の Rust test は file を読み、format ではなく decision
field を比較します。これは高速な offline regression であり、V1 を取得・実行しません。

Gate B には別の executable boundary があります。`v1-reference.lock` が正確な V1
reference commit を固定します。専用 CI job はその commit を checkout し、
`AI_COCKPIT_V1_ROOT` を設定して
`cargo test -p cockpit-core --test v1_oracle -- --ignored` を実行します。Test は
checkout identity を検証してから、`v1_oracle.py` が 14 fixtures に対して V1
governance primitive を呼び出し、decision state、blockers、unknowns、safe actions、
required checks、authority、outcome state を比較します。Adapter は
`expected.json` を読まないため、mismatch は Rust result の再投影ではなく独立した
evidence です。

外部 V1 Runtime と Python は conformance test dependency のみです。Rust binary
には link されず、adopter repository に attach されません。
