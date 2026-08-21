---
author: AI Cockpit maintainers
title: "Repository Protocol v1"
description: "Repository-owned storage、identity、receipt、decision の normative contract。"
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - protocol_v1
---

# Repository Protocol v1

Repository Protocol v1 は application repository と外部 AI Cockpit runtime の間の、
repository が所有する安定した storage boundary です。facts、decisions、evidence、
generated knowledge を保存しますが、runtime は install しません。

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
│   ├── <work-item>.verification.json
│   └── reuse/
│       ├── index.lock
│       ├── index.json
│       └── receipts/<64 lowercase hex>.json
└── knowledge/
```

`cockpit.toml` は protocol version と repository identity を持ちます。`project.json` は
attached Living Project Profile です。Work Item は scoped intent、contract、summary、outcome
を持ちます。Verification evidence は `.ai/evidence` に保存し、cross-process reusable receipt
は `reuse` store に content-addressed で保存します。Knowledge は deterministic projection で、
第二の fact source ではありません。

reuse index は schema version 1 で、`repositoryId`、`profileDigest`、`nodeId` から receipt ID への
map を binding します。receipt filename は canonical `sha256:<64 hex>` ID の lowercase hex 部分を
使うため、platform 間で path を扱えます。writer は `index.lock` を保持し、`index.pending` を
使って index を commit します。reader は uncertain、malformed、oversized、symlink、binding
不一致の store を拒否します。runtime-managed store file を adopter が手編集してはいけません。

## Identity-bearing record

Contract、verification evidence、archive manifest、reusable receipt が decision を repository
state に bind する必要がある場合、次の field を使います。Contract は次を記録します。

| Field | 意味 |
| --- | --- |
| `protocolVersion` | runtime が理解する protocol major。 |
| `repositoryId` | target repository の安定した identity。 |
| `repositorySnapshotDigest` | decision に使った repository state。 |
| `baseRevision` / `headCommit` | 利用可能な場合の source range。 |
| `projectProfileDigest` | authorization に使った attached/calibrated profile。 |
| `createdAt` | UTC RFC 3339 の作成時刻。 |

Runtime が生成する verification evidence は runtime version/digest、command result、output
identity、reuse metrics、final snapshot も記録します。Knowledge projection や human decision
receipt などはそれぞれの schema を持ち、表の全 field を暗黙に持つわけではありません。

Digest は `sha256:<64 lowercase hexadecimal characters>` です。入力は canonical JSON とし、map
key は sort、array は semantic order を保持、timestamp は UTC RFC 3339 とします。

## Reusable receipt schema

Reusable receipt は schema version 2 で、unknown field を拒否します。stable field は `receiptId`、
`nodeId`、`passed`、`outputDigest`、作成/期限切れ epoch seconds、`EvidenceContext` です。Context
は content、base/head と changed-path digest、environment、command、scope、governance、toolchain、
policy、profile、stage、runner を bind します。Receipt ID は canonical receipt body の digest です。
改ざん、failed、expired、future timestamp、binding mismatch は候補を `unknown` にし、実行します。

Store の index read は 8 MiB、reusable receipt read は 1 MiB に制限します。これは fail-closed の
resource boundary であり、任意サイズの output を保持する約束ではありません。

## Contract envelope

Contract は intent と effect boundary を認可します。scope、out-of-scope、risk、authority、acceptance、
required evidence、base revision、project profile digest、repository snapshot digest を記録します。
test 数、helper file、class 名などの中間実装詳細は凍結しません。

## Decision states

- `green`: required evidence が bounded next action を支える。
- `yellow`: evidence または capability に調査または human confirmation が必要。
- `red`: required control の失敗、authority 欠落、または invalid state。

`unknown` evidence は pass と解釈しません。Human decision は decision として記録し、独立した
verification evidence の代替にはしません。

## Evolution

- L0 content evolution は自動吸収。
- L1 verification evolution は既存 verification graph を拡張。
- L2 capability evolution は Yellow candidate と Profile proposal を生成。
- L3 governance evolution は human decision を要求し、明示的確認なしに mandatory gate になりません。

## Compatibility

現在の runtime は protocol major version 1 を受け入れ、repository material を実行する前に malformed
または未対応 version を拒否します。required field は record を消費する operation が検証します。
Optional capability を黙って upgrade したり pass に変換したりしません。未対応 request は明示的な
error、unknown、または stop になります。Protocol major migration は個別に review し、旧 evidence
を保持します。
