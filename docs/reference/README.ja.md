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
- [Agent ワークフローとレビュー境界](agent-workflow.ja.md) — Work Item、Outcome、release、安全規則の本 project 向け適用。
- [Verification route](verification-route.ja.md) — 型付き stage、直交する tier/assurance、計画、レシート、CI 境界。
- [最終置換 acceptance](final-replacement-acceptance.ja.md) — 再現可能な conformance とコピーなしの境界。
- [Repository Protocol v1](../protocol/v1/specification.ja.md) — normative storage と receipt contract。

[Reference source parity](reference-parity.ja.md) は maintainer/reviewer 向けの比較資料です。
明示的な truth state を使い、adopter route の代わりでも実装履歴をコピーする許可でもありません。
