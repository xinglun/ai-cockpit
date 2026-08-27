---
author: AI Cockpit maintainers
title: "WI-331 — checks catalog と CI/release evidence"
workItemId: WI-331-checks-release-evidence
description: "pinned checks と CI/release evidence 文書を比較し、Rust-native な responsibility boundary を記録する。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-331-checks-release-evidence
terminalArchive: .ai/work-items/archive/WI-331-checks-release-evidence.contract.json
terminalVerification: .ai/evidence/WI-331-checks-release-evidence.verification.json
terminalFinalization: .ai/decisions/WI-331-checks-release-evidence.finalize.36c6617937511f6d1d30511c3e83a25ba9717d7713f8d51a9e153b1cd7cb0281.json
terminalDecision: .ai/decisions/WI-331-checks-release-evidence.close.json
capabilityClaims:
  - reference_parity
---

# WI-331 — checks catalog と CI/release evidence

## Intent と boundary

この Work Item は commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の次の二つの pinned
reference path を一つずつ比較します。

| Pinned source path | Target responsibility |
| --- | --- |
| `docs/reference/checks-catalog.md` | `docs/reference/checks-catalog.*` が Runtime、workspace、conformance、release check を説明します。source Make/Python execution は copy しません。 |
| `docs/reference/ci-release-evidence.md` | `docs/reference/ci-release-evidence.*`、versioned gate manifest、CI/Release workflow、adopter harness が provider-derived evidence と ownership を説明します。 |

Target は shared external Rust Runtime、repository-local `.ai/` state、明示的な `--repo` context
を維持します。これは semantic responsibility parity であり、source command、wire、byte parity
ではありません。Local check、hosted provider evidence、public Release evidence、enterprise
assurance は分離されます。

## Acceptance

1. 二つの pinned path が inventory に classification、target counterpart、evidence-backed reason とともに個別登録されます。
2. English、Simplified Chinese、日本語の target page が同じ check layer、profile 選択、CI/Release evidence、failure boundary を説明します。
3. Verification coverage と Evidence Assurance を分け、local/staged result を provider/enterprise proof に昇格させません。
4. Source Makefile、Python/V1 executor、provider-global configuration、generated lifecycle truth を copy/手編集しません。
5. Inventory/documentation regression と `migrate-gap` 不在を確認し、Runtime verification、reviewed PR、merge、finalization、close、正確な branch/worktree cleanup が terminal evidence を提供します。

[English](WI-331-checks-release-evidence.md) · [简体中文](WI-331-checks-release-evidence.zh-CN.md)
