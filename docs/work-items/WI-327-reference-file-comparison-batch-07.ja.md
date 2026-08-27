---
author: AI Cockpit maintainers
title: "WI-327 — reference file comparison batch 07"
workItemId: WI-327-reference-file-comparison-batch-07
description: "pinned reference の adopter、calibration、long-cycle 文書 9 path を比較し、証拠付き Rust-native boundary を登録します。"
audience:
  - adopter
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-327-reference-file-comparison-batch-07
---

# WI-327 — reference file comparison batch 07

## Intent と境界

固定した reference commit
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` の次の 9 path を一つずつ比較します。
adopter-facing calibration、evidence、long-cycle governance semantics を保ち、source の
Python、Make、fixture、scanner、internal progress implementation は copy しません。

共有 Rust Runtime は外部に一つだけ install し、すべての repository request は明示的な
`--repo` に bind します。この batch は documentation と conformance ledger に限定され、
Runtime behavior や source wire compatibility を追加主張しません。

## file ごとの比較

| pinned reference path | Classification | Rust/adopter の対応と境界 |
| --- | --- | --- |
| `docs/reference/adopter-long-cycle-validation.ja.md` | `implemented-different-by-design` | Published binary の adopter/upgrade acceptance と日本語 lifecycle/security route が isolated install、lifecycle、rollback、cleanup evidence を保ちます。source multi-stack fixture と Make/Python orchestration は copy しません。 |
| `docs/reference/adopter-long-cycle-validation.md` | `implemented-different-by-design` | Published binary の adopter/upgrade acceptance と lifecycle/security route が isolated install、lifecycle、rollback、cleanup evidence を保ちます。source multi-stack fixture と Make/Python orchestration は copy しません。 |
| `docs/reference/adoption-reality-report.md` | `implemented-different-by-design` | Runtime capability/profile/status projection と immutable adopter receipt が template capability、adopter execution、provider evidence、enterprise assurance を分離します。 |
| `docs/reference/bandit-synchronization-security-audit.md` | `reference-only` | Source 固有の historical Bandit finding と digest は target evidence ではありません。target に Python/Bandit surface はなく、Rust-native quality/threat-model boundary は別に管理します。 |
| `docs/reference/calibration-inventory.md` | `implemented-different-by-design` | Repository-bound profile proposal/confirmation、capability/status projection、explicit unknown が fact/evidence boundary を保ち、source Python inventory は copy しません。 |
| `docs/reference/calibration-profiles.ja.md` | `implemented-different-by-design` | 日本語 calibration guide と strict JSON profile policy が累積 Lite/Standard/Strict control、人の選択、単調な upgrade、明示的 downgrade evidence を保ちます。 |
| `docs/reference/calibration-profiles.md` | `implemented-different-by-design` | Calibration guide と strict JSON profile policy が累積 Lite/Standard/Strict control、人の選択、単調な upgrade、明示的 downgrade evidence を保ちます。 |
| `docs/reference/calibration-profiles.zh-CN.md` | `implemented-different-by-design` | 中国語 calibration guide と strict JSON profile policy が累積 Lite/Standard/Strict control、人の選択、単調な upgrade、明示的 downgrade evidence を保ちます。 |
| `docs/reference/calibration-session-model.ja.md` | `implemented-different-by-design` | Explicit な proposal、confirmation、repository-bound fact が source internal Session model に対応します。汎用 interactive Session や checklist authority は導入しません。 |

## Adopter feedback の境界

Cursor adopter report は external validation input であり、新しい source wire contract ではありません。
Current Runtime v0.2.33 は stable lifecycle stdout JSON、人向け `work-item outcome`、close-before-next
entry gate、fail-closed な start/verification binding を提供します。Cursor は repository-local adapter を
明示的に install し、persist された handoff を再生します。IDE が stderr を chat に展開することは Runtime
から強制できません。Diagnostic remediation、close-gap convenience command、automatic controls
scaffold は別の product decision であり、この WI では実装済みと主張しません。

## Out of scope

Runtime command の追加、source Python/Make/YAML/fixture/Bandit file の copy、`Makefile.ai` の必須化、
global Agent/MCP 設定の変更、任意の host panel、controls scaffold、close-gap convenience は含みません。
Pinned source/target commit も変更しません。

## Acceptance と evidence

1. 9 つの pinned path をすべて読み、各 path に non-empty で証拠に基づく inventory record が一つだけあります。
2. Generated inventory はこの Work Item に `implemented-different-by-design` 8 件と `reference-only` 1 件を登録し、deferred/migrate gap を残しません。
3. English、簡体中文、日本語の comparison/parity page と本 Work Item は source pin、分類、境界で一致します。
4. Source 固有の fixture/scanner count、internal progress、未実行の provider/enterprise assurance を current Runtime の事実として表示しません。
5. Installed Runtime inspect/status/doctor/agent doctor、docs/conformance、lifecycle closure、hosted CI、正確な cleanup が terminal evidence を提供します。Historical evidence は書き換えません。

[English](WI-327-reference-file-comparison-batch-07.md) · [中文](WI-327-reference-file-comparison-batch-07.zh-CN.md)
