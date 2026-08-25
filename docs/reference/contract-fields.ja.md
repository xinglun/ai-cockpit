---
author: AI Cockpit maintainers
title: "Contract と Summary の fields"
description: "Work Item Contract と Summary を current Rust Runtime に対応付ける field reference。"
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - contract_field_mapping
---

# Contract と Summary の fields

このページは reference source の Contract と Summary の概念を current Rust Runtime に
対応付けます。これは field mapping であり、別の schema でも、reference の全 field が
実装済みだという宣言でもありません。Runtime の repository protocol state は `.ai/` に
保存し、実行ファイルは governed repository の外に一つだけ install します。

Status の意味：

- **Implemented** — current Runtime の boundary が読み取り、書き込み、または検証し、repository-local の意味が定義されている。
- **Partial** — 表現または読み取りはできるが、reference の広い semantics は保証しない。
- **External** — Agent host、provider、organization、または別システムの責任であり、Runtime は evidence を bind/display するだけで生成しない。

## Work Item Contract（`*.contract.json`）

| Field | Rust Runtime mapping | Status |
| --- | --- | --- |
| `protocolVersion` | Repository Protocol version。現在は `1`。 | Implemented |
| `contractVersion` | typed Contract V2 の optional opt-in。historical protocol record は読み取り可能。 | Implemented |
| `repositoryId` | attach した repository から導く identity。isolation に必須。 | Implemented |
| `workItemId`、`mode`、`state`、`createdAt` | Work Item identity と lifecycle metadata。 | Implemented |
| `intent`、`goal` | human-owned purpose。`intent` は legacy text または structured `businessGoal`、`userGoal`、`problem`、`constraints`、`nonGoals`、`rationale` を取れる。 | Implemented |
| `scope`、`outOfScope` | repository-relative implementation boundary。不安全または曖昧な path は fail-closed。 | Implemented |
| `risk`、`authority` | preflight が使う declaration。repository record は人の identity を認証しない。 | Implemented / External identity boundary |
| `acceptanceCriteria` | human-owned acceptance。`A1:` のような番号付き criteria は Summary evidence に bind できる。 | Implemented |
| `requiredEvidenceClasses` | lifecycle completion に必要な evidence class。 | Implemented |
| `sources` | legacy string または typed `{path, reason}` reference。 | Implemented |
| `verification` | legacy verification string または typed `{check, required}` declaration。fresh execution の代わりにはならない。 | Implemented |
| `baseRevision` | Work Item の開始 revision。snapshot から導く。 | Implemented |
| `projectProfileDigest`、`repositorySnapshotDigest` | project profile と repository snapshot の content binding。 | Implemented |
| `baseCommit`、`baselineDirtyPaths` | V2 lineage と開始時に観測した既存 dirty path の fingerprint。両方ある場合 `baseCommit` は `baseRevision` と一致する必要がある。 | Implemented |
| `archiveSequence`、`resumeHistory` | 正の archive 順序と、closed predecessor を連続して記録する lineage。 | Implemented |
| `synchronizationCheckpoint`、`synchronizationHistory` | 明示的に authorized された base synchronization と digest-bound rebase history。不完全な entry は fail-closed。 | Implemented |
| `guidelines`、`preReviewWarnings`、`acceptance` | human-authored guidance、review warning、optional alias。`acceptance` は `acceptanceCriteria` と一致する必要がある。 | Implemented |
| `authorityEvidence`、`restrictedWriteApproval`、`destructiveChangePolicy.approvalEvidence` | typed repository-local provenance と approval payload。V2 は malformed/unknown nested field を拒否し、legacy provider extension は読み取り可能。 | Implemented / External identity boundary |
| `problemStatement`、`riskAssessment`、`agentCapability`、`executionDecision` | strict typed な optional V2 safety/review input。non-continue decision は preflight を止める。 | Implemented |
| `destructiveChangePolicy`、`rollbackNote`、`unknowns`、`notCodable` | explicit safety、recovery、unresolved state declaration。 | Implemented |
| `scenarioCoverage` | optional high-risk scenario projection。required/unverified scenario は checkpoint 前に fail-closed。 | Implemented |
| `concurrencyBoundary` | parallel Work Item の Contract-owned path boundary と slot authorization。 | Implemented |
| `checkpointPolicy`、`humanDecisionPoints`、`documentationImpact`、`performanceImpact`、`governanceProfile` など | current typed validator が behavior を定義する場合だけ意味を持つ additive value。generic field は approval ではない。 | Partial |

`authority: authorized` は repository-local declaration です。enterprise identity、provider verification、organization policy、approval authenticity は外部 evidence であり、Contract bytes から推測しません。

## Change Summary（`*.summary.json`）

| Field | Rust Runtime mapping | Status |
| --- | --- | --- |
| `workItemId`、`repositoryId`、`mode`、`state` | Contract/repository binding と serial lifecycle state。 | Implemented |
| `changedPaths` | scope と archive check に使う snapshot-observed paths。 | Implemented |
| `checkpointCount` | current lifecycle の exactly-one checkpoint gate。 | Implemented |
| `preflightState`、`preflightAt`、`preflightContractDigest`、`preflightDecisionDigest`、`preflightRepositorySnapshotDigest` | repository-bound preflight decision と freshness binding。 | Implemented |
| `scenarioCoverage` | Contract と照合する Summary scenario status、evidence、reason。 | Implemented |
| `acceptanceEvidence` | stable acceptance ID と explicit evidence、intent alignment の mapping。 | Implemented |
| `intentAlignment` | optional resolved/unresolved projection。欠落は unknown のまま。 | Implemented |
| `finalDimensions` | exact twenty dimensions の receipt。decision は `GO`、`CONDITIONAL_GO`、`NO_GO`。`fourPillarProjection` は表示用。 | Implemented |
| `verification` | Runtime execution receipt は `.ai/evidence/` に書き、path の存在だけでは満たさない。 | Implemented |
| `outcome`、archive manifest、human decision | `.ai/work-items/archive/` と `.ai/decisions/` の Runtime-generated terminal projection。 | Implemented |
| `reviewReadiness`、`residualRisks`、`knownGaps`、`followUps`、`documentationAlignment` | reference として有用だが、current Runtime の universal typed Summary contract ではない。 | Partial |
| provider、enterprise、hosted-CI、attestation、SBOM、organization approval | delegated evidence として import/link できるが、Runtime は provider authority を生成しない。 | External |

## Boundary

Runtime は Contract の source language を保持し、governance fact を machine-translate しません。
Outcome localization は label と presentation だけを変えます。missing、stale、contradictory、malformed、
identity-mismatched field は適用される gate に従って yellow または red のままで、documentation projection で green にはなりません。

[Reference source parity](reference-parity.ja.md) と [Commands](commands.ja.md) も参照してください。

## Contract review の境界

現在の Rust 境界は、ガバナンス評価の前に任意の `scenarioCoverage` リストの
形を検証します。各項目には `scenario`、boolean の `required`、対応する
status、evidence リストが必要です。`verified` には evidence、
`not_applicable` には reason が必要で、重複名や未知の nested field は
fail closed になります。これは構造検証だけです。シナリオを必須にするかは
risk policy が決め、Runtime がシナリオ、期待結果、verification plan を作る
ことはありません。

`acceptanceCriteria` は空でない人間の宣言でなければなりません。`A<n>:` の
番号付き criteria は Summary の evidence mapping を opt-in する形式として
残し、番号なしの criteria は legacy/原文の宣言として読み取り可能です。
`concurrencyBoundary` も schema、正の容量、空でない理由を検証してから
parallel slot を使います。これらの検証は verification tier を assurance
に変換せず、slot 宣言を権限決定にも変換しません。
