---
author: AI Cockpit maintainers
title: "WI-388 — reference documentation batch 21"
workItemId: WI-388-reference-documentation-batch-21
description: "troubleshooting、adoption stability、threat model の pinned 6 文書を比較し、source authority を copy せず bounded Rust-native parity を記録する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-388-reference-documentation-batch-21
terminalArchive: .ai/work-items/archive/WI-388-reference-documentation-batch-21.contract.json
terminalVerification: .ai/evidence/WI-388-reference-documentation-batch-21.verification.json
terminalFinalization: .ai/decisions/WI-388-reference-documentation-batch-21.finalize.796631a3301dfcc04a7ef0e0381c01f3d8fca7bffbf9278763ea588a53bbc5d4.json
terminalDecision: .ai/decisions/WI-388-reference-documentation-batch-21.close.json
---

# WI-388 — reference documentation batch 21

## Intent と boundary

Pinned source commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の次の 6 path を一つずつ読みます。現在の Rust-native documentation route で reader-facing な governance meaning を保持し、source command、provider authority、historical stability claim は target に copy しません。

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/security/threat-model.md` | implemented-different-by-design | 三言語 `docs/security/threat-model.*` が asset、trust boundary、fail-closed threat、external control limit を保持します。全 malicious intention の検出や enterprise certification は主張しません。 |
| `docs/template-adopter-stability-matrix.md` | implemented-different-by-design | `docs/reference/final-replacement-acceptance.md`、`docs/getting-started/standard-adoption-guide.md`、`docs/reference/ci-release-evidence.md`、adopter harness が evidence-kind と adoption boundary を分担します。template-only evidence は external stability proof ではありません。 |
| `docs/troubleshooting.md` | implemented-different-by-design | 三言語 `docs/reference/troubleshooting.*` が stop state、recovery、evidence preservation を提供し、compatibility-only redirect にはしません。 |
| `docs/troubleshooting/installation.ja.md` | implemented-different-by-design | 日本語 install、strict verification、troubleshooting page が uncertainty stop と explicit attachment を保持します。 |
| `docs/troubleshooting/installation.md` | implemented-different-by-design | English install、strict verification、troubleshooting page が uncertainty stop、immutable artifact check、explicit attachment を保持します。 |
| `docs/troubleshooting/installation.zh-CN.md` | implemented-different-by-design | 中国語 install、strict verification、troubleshooting page が recovery と repository context boundary を保持します。 |

## Acceptance

- 各 pinned file を読み、inventory classification と counterpart mapping を明示します。
- 三言語 comparison、parity、Work Item record が一致し、`migrate-gap` は 0 のままです。
- source Python/Make command、provider authority、historical evidence を copy/promotion しません。
- shared Runtime と object/adopter 継承 boundary（一つの installed binary、明示的 `--repo`、分離された repository fact/evidence）を明示します。
- documentation、inventory、governance、installed Runtime verification が pass します。

## Verification と non-claims

これは semantic/documentation parity であり、source command、JSON-wire、provider state の compatibility ではありません。同名 file がなくても bounded counterpart と non-claim が記録されていれば omission ではありません。

[English](WI-388-reference-documentation-batch-21.md) · [简体中文](WI-388-reference-documentation-batch-21.zh-CN.md)
