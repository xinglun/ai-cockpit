# Conformance harness

各 case には repository material、contract、evidence、明示的な governance input、期待する
semantic result があります。通常の Rust test は file を読み、format ではなく decision
field を比較します。これは高速な offline regression であり、V1 を取得・実行しません。

Gate B は二つの明示的な boundary に分かれます。Hosted CI は committed offline
semantic corpus だけを実行し、reference repository にはアクセスしません。今後の
file-by-file comparison は `AI_COCKPIT_REFERENCE_ROOT` で指定する local Git checkout
を使い、`reference-source.lock` の commit に固定します。checkout は clean で HEAD が
lock と一致しなければなりません。`reference_source_policy.py` は clone/fetch なしで
これを検証します。legacy の executable V1 oracle は maintainer が `v1-reference.lock`
と exact local checkout を使って local のみで実行し、Hosted CI の dependency ではありません。

local comparison では `AI_COCKPIT_REFERENCE_ROOT` を maintainer の checkout に設定し、次を実行します。

```bash
python3 tests/conformance/reference_source_policy.py \
  --lock tests/conformance/reference-source.lock \
  --reference "$AI_COCKPIT_REFERENCE_ROOT"
```

legacy V1 Runtime と Python は optional な local conformance dependency のみです。Rust binary
には link されず、adopter repository に attach されません。
