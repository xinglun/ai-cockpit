---
author: AI Cockpit maintainers
title: "WI-437 — local-reference governance rebaseline delta"
workItemId: WI-437-reference-rebaseline-governance
description: "maintainer 管理の local reference checkout で変更された governance file 7 件を再確認する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-437-reference-rebaseline-governance
terminalArchive: .ai/work-items/archive/WI-437-reference-rebaseline-governance.contract.json
terminalVerification: .ai/evidence/WI-437-reference-rebaseline-governance.verification.json
terminalFinalization: .ai/decisions/WI-437-reference-rebaseline-governance.finalize.json
terminalDecision: .ai/decisions/WI-437-reference-rebaseline-governance.close.json
---

# WI-437 — local-reference governance rebaseline delta

この documentation/conformance Work Item は、以前の ledger 後に source bytes が変化した 7 file を
一つずつ再確認します。semantic reference は
`/Users/sei-rinn/dev/workspace_python/ai-cockpit-template` の local checkout とし、public reference
repository には接続しません。Python、Make、YAML、source JSON artifact は Rust project にコピーしません。

[English](WI-437-reference-rebaseline-governance.md) · [简体中文](WI-437-reference-rebaseline-governance.zh-CN.md)

## Scope

- `.ai/cockpit/README.md`、`.ai/cockpit/README.ja.md`、`.ai/cockpit/adoption.ja.md`、
  `.ai/guards/changed_critical_coverage_policy.json`、`.ai/guards/coverage_policy.yaml`、
  `.ai/quality/governance-routing.yaml`、`.ai/schemas/task_outcome.schema.json` を pinned local source
  commit で再読する。
- 各 file に Rust-native counterpart または non-portability reason を登録する。
- machine inventory、三言語 comparison/parity docs、regression assertion だけを更新し、Runtime behavior は変更しない。

## File-level decision

7 件すべてを `implemented-different-by-design` とします。Source の変更は Python/Make surface の cleanup
です。obsolete な `REPORT_LANGUAGE` argument、Python-only coverage association、重複した gate metadata、
Python Task Outcome schema の template fields が整理されました。Rust は独自の typed OutcomeV2/humanHandoff
と dynamic gate boundary を保持し、source wire shape は compatibility requirement としません。

## Verification

Local source policy、inventory regression、documentation acceptance、parity status、governance integrity gate、
Runtime verification をすべて通過させます。ledger は `previousBatch`、`previousClassification`、
`sourceChangedSincePrevious` provenance を保持し、7 件の current record を `deferred-next-batch` から解決します。
