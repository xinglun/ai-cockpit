---
author: AI Cockpit maintainers
title: "Configuration reference"
description: "Repository-owned TOML configuration、profile state、generated Work Item file。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - configuration
---

# Configuration reference

Repository configuration format は TOML です。JSON には変更しません。

## `.ai/cockpit.toml`

`attach` は次の最小 file を作ります。

```toml
protocol_version = 1
repository_id = "sha256:<64 lowercase hexadecimal characters>"
```

`repository_id` は最初の attach 時に生成され、以後の request は repository-owned file から読み取ります。
absolute path の hash ではないため、attach 済み repository を移動しても evidence は別 repository になりません。
runtime は両方を validate し identity mismatch を拒否します。runtime source や V1 file を `.ai/` に copy しません。

## `.ai/agent-interface.json`

`attach` は strict な repository-local discovery manifest も書きます。`schemaVersion`、`protocolVersion`、stable
`repositoryId`、`rootBinding: "manifest-parent"`、Runtime capability、`adapterState: "unconfigured"` を持ちます。
これは discovery fact であり、provider instruction、authorization、global MCP configuration ではありません。Provider install は
`attach` とは別の明示操作です。

## `.ai/adapters/<provider>.json`

`agent install` は provider、repository ID、repository-relative target、adapter version、managed section の digest を含む strict ownership record を書き込みます。
`doctor`、`repair`、`detach` はこれを ownership の根拠にし、record の欠落、変更、重複、identity mismatch は conflict として扱います。
ここには global Agent/MCP configuration を保存しません。

## `.ai/project.json`

`attach` は `state: "calibration_required"` の attached profile を作ります。`profile confirm` 後に profile version が
増え、選択した quality command が verified として記録されます。wrapper は `profileVersion`、`repositoryId`、`state`、
`profileDigest`、`tests`、`buildSystems` を持ちます。unknown profile field は拒否されます。

## `.ai/policy.json`

Enterprise adopter は TOML の設定形式を変更せず、strict な policy document を
任意で有効にできます。

```json
{
  "schemaVersion": 1,
  "organization": {
    "policyId": "org-release-v1",
    "layer": "organization",
    "rules": [{
      "operation": "release",
      "approvalMode": "single_authorized_human",
      "requiredEvidence": ["delegated:github"]
    }]
  }
}
```

Project layer は要求を追加できますが、organization layer を弱化できません。
Work Item contract には `layer: "work_item"` の `governancePolicy` を含められます。
すべての policy object は unknown field を拒否します。`attach` は policy を生成
しません。Policy は governance decision であり scaffold ではないためです。

External proof は `evidence import` で別途取り込みます。Metadata JSON は strict な
`DelegatedEvidence` object、raw file は bytes 単位で digest 化されます。`evidence list`
（または MCP `delegated_evidence_list`）で bind 済み receipt を確認できます。

## Work Item record

`start` は `.ai/work-items/active/` に次を生成します。

- `<id>.contract.json` — intent、scope、authority、acceptance、required evidence、base revision、profile digest、repository snapshot digest。
- `<id>.summary.json` — lifecycle state、ちょうど 1 回の checkpoint receipt、repository/Contract に bind された preflight
  decision（`preflightState`、decision digest、snapshot digest、timestamp）。

serial lifecycle は fail-closed です。Work Item は non-red の preflight を記録してから 1 回だけ checkpoint を行い、
verification 完了時に decision を refresh します。`finish` には green の結果が必要で、重複 checkpoint や順序外の
finish/archive/close は拒否されます。失敗した transition の active record は復旧のため保持されます。

`work-item new --repo <path> --id <id> --mode <mode>` は同じ contract writer を使って `not_ready` skeleton を作ります。自動入力は
4 つの deterministic fact（`repositoryId`、`baseRevision`、`projectProfileDigest`、`repositorySnapshotDigest`）だけで、intent、scope、
acceptance criteria、authority は空または `unknown` のままです。`profile propose` は candidate amendment を出力するだけで、formal
profile の bytes/digest を変更しません。

`verify --work-item <id>` は `.ai/evidence/<id>.verification.json` を書きます。`finish` は outcome、`archive` は archive manifest、
`close` は human decision を記録します。green に見せるため手編集してはいけません。

Cross-process reusable evidence は runtime が `.ai/evidence/reuse/` で管理します。schema、identity binding、resource limit は
[Protocol v1](../protocol/v1/specification.ja.md) を参照してください。
