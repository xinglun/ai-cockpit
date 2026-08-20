# Repository Protocol v1

Repository Protocol v1 は application repository と外部 AI Cockpit runtime の間の、
repository が所有する安定した storage boundary です。facts、decisions、evidence、
generated knowledge を保存しますが、runtime を install しません。

## Layout

```text
.ai/
├── cockpit.toml
├── project.json
├── work-items/
│   ├── active/
│   └── archive/
├── decisions/
├── evidence/
└── knowledge/
```

`cockpit.toml` は protocol version と repository identity を持ちます。`project.json`
は現在の Living Project Profile です。Work Item は scoped intent と outcome を
持ちます。Evidence は content-addressed receipt または delegated provider evidence
への参照です。Knowledge は deterministic projection であり第二の fact source では
ありません。

## 必須 identity

すべての protocol-bound record は `protocolVersion`、`repositoryId`、
`repositorySnapshotDigest`、`createdAt` を持ちます。Runtime が生成する evidence は
`runtimeVersion` と `runtimeDigest` も持ちます。Historical record は decision boundary
で使った Project Profile digest を保持します。

Digest は `sha256:<64 lowercase hexadecimal>` です。入力は canonical JSON とし、map
key は sort、array は semantic order を保持、timestamp は UTC RFC 3339 とします。

## Contract envelope

Contract は intent と effect boundary を認可します。scope、out-of-scope、risk、
authority、acceptance、required evidence、base revision、project profile digest、
repository snapshot digest を記録します。test 数、helper file、class 名などの中間
実装詳細を凍結しません。

## Decision states

- `green`: required evidence が bounded next action を支える。
- `yellow`: evidence または capability に調査または human confirmation が必要。
- `red`: required control の失敗、authority 欠落、または invalid state。

`unknown` evidence は pass と解釈しません。Human decision は decision として記録し、
独立した verification evidence の代替にしません。

## Evolution

- L0 content evolution は自動吸収。
- L1 verification evolution は既存 verification graph を拡張。
- L2 capability evolution は Yellow candidate と Profile proposal を生成。
- L3 governance evolution は human decision を要求し、明示的確認なしに mandatory gate
  になりません。

## Compatibility

protocol major version 1 をサポートしない実装は Red で停止します。Runtime upgrade は
protocol 1 を継続サポートする限り repository file を変更しません。Protocol major
migration は個別にレビューする操作です。

