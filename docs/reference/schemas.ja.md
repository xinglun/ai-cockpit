---
author: AI Cockpit maintainers
title: Schema と record authority
description: AI Cockpit の Rust-native record map と validation boundary。
audience:
  - adopter
  - contributor
  - maintainer
  - auditor
status: current
authority: translation
canonical: docs/reference/schemas.md
lastVerifiedBy: WI-378-reference-documentation-batch-17
capabilityClaims:
  - typed_record_schemas
---

# Schema と record authority

[English](schemas.md) · [简体中文](schemas.zh-CN.md) · [日本語](schemas.ja.md)

Record の有効性は executable Rust Protocol と repository validator が決めます。文書や example は境界を説明するだけで、authority を付与しません。repository-bound record には repository identity と、必要に応じて Work Item または snapshot の binding が含まれます。

| Record / surface | Rust-native authority | Boundary |
| --- | --- | --- |
| Work Item Contract | `cockpit-protocol` の typed Contract と repository validation | 人の intent、scope、authority、acceptance、verification を推測しません。 |
| Change Summary | `.ai/work-items/` 下の Runtime-generated Summary | changed paths、checkpoint、preflight、acceptance evidence、cost facts は derive/bind されますが、Summary は変更を認可しません。 |
| Project Profile | `.ai/project.json` と profile policy | detection fact と human confirmation を分離し、candidate proposal は baseline を変更しません。 |
| Repository Protocol | `.ai/cockpit.toml`、`project.json`、attached identity | Runtime に persistent current repository や global Work Item はありません。 |
| Verification Evidence | `.ai/evidence/<work-item>.verification.json` | schema、Work Item、repository、snapshot、runtime、receipt、`passed` を検証します。file の存在だけでは evidence になりません。 |
| Checkpoint Evidence | Summary の typed `checkpointEvidence` | stage、順序、hash、count、amendment、resume freshness は fail closed です。 |
| Delegated Evidence | `evidence import` metadata と raw-byte digest | Provider/enterprise assurance は外部の責任です。import した bytes を bind/display しますが、発明しません。 |
| Archive / decision | archive manifest、finalization receipt、close decision | immutable history と human-decision boundary であり、編集可能な status cache ではありません。 |
| Outcome / status | Runtime projection（`work-item outcome`、`status`） | 派生 view は merge、release、approval を認可しません。 |
| Audit export | `audit export` event bundle | 長期の immutable retention は外部 SIEM/WORM/retention system が担当します。 |

## Strictness と compatibility

現在の V2 record は、必須 field の破損、安全でない path、重複 identity、strict typed schema の未知 nested field、stale snapshot、cross-repository evidence を拒否します。Legacy record は immutable のまま保全し、現在の identity 要件を満たさない場合は historical/unknown として投影します。暗黙の書き換えや in-place upgrade は行いません。

Rust record は reference の責務と semantic に対応しますが、直接の JSON-wire または Python-module compatibility ではありません。reference の `.ai/project_profile.yaml`、`.ai/cockpit/checks.yaml`、generated status、source-specific registry は、Rust-native counterpart が明記されない限り比較材料です。
