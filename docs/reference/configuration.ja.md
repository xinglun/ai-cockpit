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

## `.ai/project/` の宣言

Adopter は repository-owned な strict JSON 宣言を 3 つ追加できます。

- `capabilities.json`: capability、non-capability、critical domain、厳密な operation-to-capability mapping。
- `success_criteria.json`: project criteria と evidence hint の表示だけを担い、Contract acceptance を置き換えません。
- `profile-policy.json`: approved boundary、critical path、review requirement、明示的な unknown。reference profile policy の
  JSON projection であり、`.ai/project.json` は identity と observed-quality profile として残ります。

各 file は regular file のみを受け付け、unknown field と duplicate JSON key を拒否し、repository ID と review 時点の
snapshot digest に bind します。`capability show` と MCP `capability_show` は semantic declaration digest を表示するだけで
書き換えません。Contract が `operation`/`requestedOperation` を明示する場合、Preflight は十分な mapping を要求します。
missing、malformed、foreign、stale、conflict の入力は yellow/unknown のままです。intent prose は mapping を満たさず、
project criterion も Work Item を approve/complete しません。

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

## Contract V2 の意味論境界

Contract は repository の `protocolVersion` を維持したまま `contractVersion: 2` を
選択できる。V2 の `intent` は `businessGoal`、`userGoal`、`problem`、`constraints`、
`nonGoals`、`rationale` を持つ object にでき、過去の一行 intent も読み取り可能である。
`sources` と `verification` も `path/reason`、`check/required` の typed object をサポートし、
legacy の string は過去 bytes の互換性のためだけに残す。

Contract の top-level と構造化フィールドは strict な unknown-field validation を使う。
duplicate JSON key、型不正、互換性のない schema は fail closed になる。宣言された unknown、
`notCodable`、Agent capability の制限、または `continue` 以外の execution decision がある
場合、preflight は `reviewState: needs_human_confirmation` と structured
`humanDecisionRequest` を返す。これは approval ではなく、Contract を補完して preflight を
再実行するまで checkpoint できない。

scenario coverage、final acceptance dimensions、parallel boundary は別の Contract extension
である。`verify --workers` は実行時 concurrency であり Work Item の parallel authorization
ではない。Contract 原文は owner の言語で保持し、Runtime は自動翻訳しない。

### Contract V2 の lineage と governance field

次の optional field は typed な Contract data です。protocol-v1 record では
省略または空のまま読み取り、V2 record で利用できます。

- `baseCommit` と `baselineDirtyPaths` は Work Item の開始 revision と開始前の dirty file
  （`path`、`status`、`fingerprint`）を bind します。legacy spelling の `baseRevision` も残します。
- `archiveSequence` は順序 metadata です。archive manifest 自身の digest binding の代わりにはなりません。
- `resumeHistory` は closed predecessor への連続した transition を記録します。old/new base、branch identity、Contract digest、
  manifest path、predecessor closure flag を各 entry に持ちます。
- `synchronizationCheckpoint` は明示的な `authorized: true` と空でない reason が必要です。
  `synchronizationHistory` は base と rebase transition を記録し、無関係な dirty path を隠すためには使えません。
- `guidelines`、`preReviewWarnings`、optional な `acceptance` は人間が記述した指示と stable な受入宣言を保持します。
  空の guideline は拒否されます。
- `authorityEvidence` と `restrictedWriteApproval` は repository-local の provenance record であり、identity authentication ではありません。
  destructive approval evidence は identity level、actor、scope、evidence payload を typed に持ち、provider/enterprise claim は外部検証の対象です。

`contractVersion: 2` の `mode` は `investigate`、`author_todo`、`code`、`review`、`cleanup` のいずれかです。
`code` Contract では `unknowns` を空にし、`notCodable: false` にする必要があります。
legacy の `mode: implementation` は Contract V2 を選択していない record だけ読み取り可能です。
不正な lineage、approval、mode、cross-field combination は Contract validation で停止します。
過去の Contract bytes に対する backfill や rewrite は行いません。

`verify --work-item <id>` は `.ai/evidence/<id>.verification.json` を書きます。`finish` は outcome、`archive` は archive manifest、
`close` は human decision を記録します。green に見せるため手編集してはいけません。

Cross-process reusable evidence は runtime が `.ai/evidence/reuse/` で管理します。schema、identity binding、resource limit は
[Protocol v1](../protocol/v1/specification.ja.md) を参照してください。
