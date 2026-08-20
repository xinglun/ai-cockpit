# バージョニング

Runtime version と Repository Protocol version は独立しています。

```text
ai-cockpit 2.3.5
supports repositoryProtocol = 1

repository:
protocol_version = 1
```

Runtime upgrade は Protocol 1 を継続サポートしたまま capability を追加できます。
Protocol 1 → Protocol 2 だけが repository migration です。Runtime startup は両方の
version と Runtime digest を報告しなければなりません。

Protocol compatibility は明示します。未対応 major protocol は Red、現在の Runtime
にない optional capability は safe action 付き Yellow です。Historical Work Item は
decision boundary で使用した Project Profile digest と protocol version を保持します。

