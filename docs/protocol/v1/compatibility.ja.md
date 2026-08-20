# Protocol compatibility ルール

互換性アルゴリズムは意図的に小さくします。

1. repository material を実行せず protocol version を parse する。
2. malformed または未対応 major version は Red で拒否する。
3. 必須 artifact field が validate された場合だけ対応 major version を受け入れる。
4. optional capability の不足は明示的 safe action 付き Yellow とする。
5. compatibility inspection 中に historical artifact を書き換えない。

Runtime は supported protocol range、repository は一つの protocol major を宣言します。
Runtime minor/patch release は migration ではありません。Major protocol migration は
新しい Work Item を作り、旧 evidence を保持し、source/target protocol version を記録します。

