---
author: AI Cockpit maintainers
title: "CI と Release Evidence"
description: "ownership を明示した provider-derived CI と public Release evidence。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: WI-331-checks-release-evidence
keywords: [ai-cockpit, ci, release, evidence]
---

# CI と Release Evidence

CI/Release record は delegated evidence です。authority は hosted provider と正確な公開
artifact にあり、PR body、Agent message、local の「passed」主張にはありません。Rust Runtime
は record を bind/validate できますが、GitHub Actions や enterprise approval system を
代行しません。

## CI evidence

versioned `tests/ci/repository_gate_manifest.json` と CI route は repository、Contract、
base revision、head revision、selected profile、ordered gate ID、route/manifest digest を
bind します。final gate report は required gate と result を一つずつ記録します。Hosted
adapter は provider workflow run、job 名、job conclusion、正確な head SHA も保持します。

Required job は明示的な集合です。skipped/failed job は aggregate を green に見せるために
省略せず、record に残します。aggregate conclusion は全 job result と failure reason に
一致しなければなりません。PR body や人間の prose は provider run の代わりにならず、local
fixture も hosted assurance に昇格できません。

Profile は policy が選び、累積します。`light`、`standard`、`strict` は verification
coverage であり assurance level ではありません。merge/release stage には strict floor が
あり、unknown path と release-owned file は strict route に fail closed します。repository-bound
decision の authority は Rust Contract gate にあり、収束期間の script runner は bounded
execution shadow です。

## Release evidence

Release workflow は version、tag、source commit、Cargo.lock digest、target archive、executable
member、checksum manifest、SBOM、provenance を bind します。各 target は期待する archive
layout を持ち、実際に公開された bytes から checksum を再計算します。SBOM/attestation subject
は同一の source と artifact identity を指さなければなりません。tag や upload だけでは stable
public Release ではありません。

Release evidence の state は分離されています。

| State | 意味 | Authorization boundary |
| --- | --- | --- |
| `candidate` | 公開前の staged source/artifact record。 | review を支えますが public Release を証明しません。 |
| `verified` | 正確な source commit と required job/asset の成功を示す provider evidence。 | 公開手順を支えますが、まだ published Release ではありません。 |
| `published` | 正確な public Release と asset set に付いた verified evidence。 | 公開事実であり enterprise certification ではありません。 |
| `failed` | provider/artifact check が失敗し、理由を含む record。 | `verified`/`published` を authorize しません。 |

Post-release adopter harness は別の acceptance receipt を生成し、download した immutable
tag/artifact、binary/archive digest、isolated repository identity、lifecycle evidence、
cleanup/isolation manifest を bind します。成功 receipt はその binary がその adopter を
governance した evidence であり、全 technology stack や全 enterprise environment の coverage
ではありません。

## Ownership と failure

Local Runtime/manifest check は repository evidence です。Hosted run/job result、merge
observation、signing、SBOM publication、attestation、branch protection、enterprise approval は
external/provider-owned evidence のままです。AI Cockpit は提供された identity、origin、assurance、
collection time、digest、validity、raw reference を記録できますが、provider result を作りません。

Missing job、aggregate から隠された skipped/failed job、head/base mismatch、artifact/SBOM digest
違い、checksum の duplicate/missing、malformed JSON、provider-bound evidence のない Release
state は fail closed です。failure receipt と source identity を保持し、published Release を
unpublished に書き換えたり、failed receipt を次の version に再利用したりしません。

Adopter project でも同じです。shared Runtime は project の外部、repository state は `.ai/`
に隔離され、すべての command は明示的な `--repo <path>` を使います。
