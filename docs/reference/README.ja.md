---
author: AI Cockpit maintainers
title: "Reference"
description: "利用者向け command、configuration、recovery reference。"
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - reference_index
---

# Reference

まず [current reader route](../current/README.ja.md) と capability walkthrough を読み、その後で参照してください。
route index は一般利用者の journey と正確な machine-facing detail を分離します。

- [Getting started](../getting-started/README.ja.md) — install と最初の attach。
- [Features](../features/README.ja.md) — capability goal と boundary。
- [Operations](../operations/README.ja.md) — lifecycle、recovery、upgrade、acceptance。

- [Command reference](commands.ja.md) — command group、required binding、output behavior。
- [Configuration reference](configuration.ja.md) — `.ai/cockpit.toml`、profile、generated record。
- [Troubleshooting と recovery](troubleshooting.ja.md) — stop state と安全な次の action。
- [人間向け Outcome](outcome-report.ja.md) — 読みやすい結果、リスク、証拠、次の action。
- [Governance Profile](governance-profiles.ja.md) — リスクに応じた Light/Standard/Strict route と assurance boundary。
- [Cockpit Status の読み方](how-to-read-cockpit-status.ja.md) — 人向けの色、証拠、次の action の読み順。
- [Agent ワークフローとレビュー境界](agent-workflow.ja.md) — Work Item、Outcome、release、安全規則の本 project 向け適用。
- [Verification route](verification-route.ja.md) — 型付き stage、直交する tier/assurance、計画、レシート、CI 境界。
- [実装 Knowledge](implementation-knowledge.ja.md) — 決定的で evidence-bound な record と query の境界。
- [Input trust data flow](input-trust-dataflow.ja.md) — provenance 分類と fail-closed な input 処理。
- [Installed Runtime lifecycle](installed-lifecycle.ja.md) — shared Runtime の install、attach、upgrade、rollback 境界。
- [Instruction traceability](instruction-traceability.ja.md) — source path、Work Item、evidence、close の関係。
- [Japanese capability assessment boundary](japanese-capability-assessment.ja.md) — 一般的な fluency を主張しない evidence-bound な多言語 coverage。
- [最終置換 acceptance](final-replacement-acceptance.ja.md) — 再現可能な conformance とコピーなしの境界。
- [Repository Protocol v1](../protocol/v1/specification.ja.md) — normative storage と receipt contract。

[Reference source parity](reference-parity.ja.md) は maintainer/reviewer 向けの比較資料です。
明示的な truth state を使い、adopter route の代わりでも実装履歴をコピーする許可でもありません。
