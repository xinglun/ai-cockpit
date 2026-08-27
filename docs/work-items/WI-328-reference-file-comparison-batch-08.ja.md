---
author: AI Cockpit maintainers
title: "WI-328 — reference file comparison batch 08"
workItemId: WI-328-reference-file-comparison-batch-08
description: "固定した calibration と capability reference の 9 path を一つずつ比較し、Rust-native boundary を記録します。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-328-reference-file-comparison-batch-08
terminalArchive: .ai/work-items/archive/WI-328-reference-file-comparison-batch-08.contract.json
terminalVerification: .ai/evidence/WI-328-reference-file-comparison-batch-08.verification.json
terminalFinalization: .ai/decisions/WI-328-reference-file-comparison-batch-08.finalize.json
terminalDecision: .ai/decisions/WI-328-reference-file-comparison-batch-08.close.json
---

# WI-328 — reference file comparison batch 08

## Intent と境界

固定した reference commit
e5acb677da6621004d96f0ef353c58fe8d3acfbf の 9 path を一つずつ比較します。
adopter-facing な calibration と capability-truth の責務を保ち、source の
Python、Make、wizard、matrix bytes は copy しません。

共有 Rust Runtime は外部に一つだけ install し、すべての repository request は
明示的な --repo に bind します。この batch は documentation と conformance
ledger に限定され、Runtime command や source wire compatibility を追加しません。

## file ごとの比較

| pinned reference path | Classification | Rust/adopter の対応と境界 |
| --- | --- | --- |
| docs/reference/calibration-session-model.md | implemented-different-by-design | Repository-bound profile proposal、human confirmation、明示的 calibration facts が fact/evidence boundary を保ちます。汎用の persisted Session や proposal の active policy 化は行いません。 |
| docs/reference/calibration-session-model.zh-CN.md | implemented-different-by-design | 中国語 reader にも同じ repository-bound proposal/confirmation boundary を示し、unknown と human authority を可視化します。 |
| docs/reference/calibration-session.ja.md | implemented-different-by-design | Source の ten-stage interactive Session は target の明示的 profile proposal/confirmation に意味だけを写します。Make/Python と enterprise/security claim は copy しません。 |
| docs/reference/calibration-session.md | implemented-different-by-design | Source の persisted ten-stage wizard は source-specific orchestration です。Target calibration は read-only-first、repository-bound で、policy change に human confirmation を要求します。 |
| docs/reference/canonical-terminology.md | implemented-different-by-design | .ai/glossary.md、configuration、Outcome reference が canonical terms を提供します。Governance light と source Calibration lite は alias ではなく、release は profile ではなく operation です。 |
| docs/reference/capability-claim-authoring.md | reference-only | Source の lexical claim checker と matrix-binding front matter は target Runtime gate ではありません。Target registry は observed repository facts と exclusion を示すだけで、将来の境界は候補 WI-329 に記録します。 |
| docs/reference/capability-evidence-freshness.md | reference-only | Rust は Work Item verification freshness と identity-bound receipt を検証しますが、Capability Truth row expiry や portable-environment matrix は提供しません。拡張は候補 WI-329 の範囲です。 |
| docs/reference/capability-truth-matrix.json | reference-only | Source の 30-row public matrix は copy しません。capability_truth_registry は request-scoped observed-capability projection であり、public claim authorization や adopter/provider proof ではありません。 |
| docs/reference/capability-truth-matrix.md | reference-only | 現在の capability/adoption page は observed fact、adopter installation、provider evidence、enterprise assurance の境界を説明します。後続の明示的な承認なしに source matrix/claim checker を主張しません。 |

4 つの reference-only は明示的な product boundary であり、未登録 omission では
ありません。候補 WI-329 はこの batch では開始しません。Rust-native claim/evidence
matrix、freshness policy、multilingual binding test、adopter docs を人が所有する
scope として定義してから開始します。Source Python/Make checker は copy しません。

## Cursor adopter feedback の照合

Cursor report は external adopter evidence であり、新しい source wire contract
ではありません。Current Runtime evidence は次をすでに提供します。

- lifecycle stdout の stable JSON と replay 可能な work-item outcome
- close-before-next entry check と明示的 readyOnBase
- dirty または unsynchronized base を拒否する fail-closed start check
- relevant change 後の verification invalidation
- repository-local Agent adapter の明示的 install と automatic chat posting の不在

Runtime は IDE chat panel の展開を強制できません。adapter/host が durable human
handoff を表示または replay します。詳細な mismatch remediation、controls
scaffold、close-gap convenience command は有用な後続作業ですが、現在の parity
として暗黙に claim しません。Target は Makefile.ai も要求せず、明示的 --repo の
CLI/MCP を adopter interface とします。

## Out of scope

Runtime behavior の追加、source Python/Make/YAML の copy、generic calibration
wizard、public claim matrix、Make integration の強制、global Agent/MCP config の
変更、pinned source/target commit の変更は行いません。

## Acceptance と evidence

1. 9 つの pinned path をすべて読み、各 path に non-empty で evidence-backed な
   inventory record が一つだけあります。
2. Inventory は WI-328 に implemented-different-by-design 5 件と reference-only
   4 件を記録し、deferred や隠れた分類を残しません。
3. English、簡体中文、日本語の comparison/parity page と本 Work Item が source
   pin、分類、Cursor boundary、候補 WI-329 follow-up で一致します。
4. Target は generic ten-stage Session、source Python/Make execution、public
   capability-claim matrix、provider identity、enterprise assurance を target
   evidence なしに claim しません。
5. Installed Runtime inspect/status/doctor/agent doctor、docs/conformance、
   lifecycle closure、hosted CI、exact cleanup が terminal evidence を提供します。
   Historical evidence は書き換えません。

[English](WI-328-reference-file-comparison-batch-08.md) · [中文](WI-328-reference-file-comparison-batch-08.zh-CN.md)
