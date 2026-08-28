---
author: AI Cockpit maintainers
title: "WI-348 — verification、operation-time policy、provider boundary の reference batch"
workItemId: WI-348-reference-verification-operation-policy
description: "固定した十個の reference path を比較し、bounded な Rust-native verification/policy gap を閉じる。"
audience: [maintainer, reviewer]
status: implemented
authority: translation
canonical: docs/work-items/WI-348-reference-verification-operation-policy.md
lastVerifiedBy: WI-348-reference-verification-operation-policy
terminalArchive: .ai/work-items/archive/WI-348-reference-verification-operation-policy.contract.json
terminalVerification: .ai/evidence/WI-348-reference-verification-operation-policy.verification.json
terminalFinalization: .ai/decisions/WI-348-reference-verification-operation-policy.finalize.json
terminalDecision: .ai/decisions/WI-348-reference-verification-operation-policy.close.json
capabilityClaims: [reference_parity, operation_time_policy_evaluation]
---

# WI-348 — verification、operation-time policy、provider boundary の reference batch

[English](WI-348-reference-verification-operation-policy.md) · [简体中文](WI-348-reference-verification-operation-policy.zh-CN.md)

## Intent と boundary

Pinned reference commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の次の十個の path を
一つずつ比較します。verification、multilingual、performance、operation-time governance
の有用な意味を Rust に保ちますが、source Python/Make Runtime、generated assessment bytes、
provider-global configuration、historical provider truth はコピーしません。

共有 external Runtime は request-scoped のままです。すべての adopter/object project は
明示的な `--repo` を使い、Contract、evidence、performance fact、decision を repository-local
に保持します。

## File-level decision

| Reference path | Classification | Decision |
| --- | --- | --- |
| `docs/reference/japanese-capability-assessment.md` | implemented-different-by-design | Source matrix を bounded な三言語 reader/Outcome/adversarial/installation/documentation check に対応付け、一般的な fluency は主張しません。 |
| `docs/reference/lightweight-verification-and-soft-gates.md` | implemented-different-by-design | 比例した route、content-bound reuse、partial dependency、単調な escalation、可視の advisory boundary を文書化します。 |
| `docs/reference/multilingual-semantic-parity.md` | implemented-different-by-design | 三言語の Runtime-owned presentation fact を揃え、Contract 値は作成言語で保持します。 |
| `docs/reference/open-pr-issue-reconciliation-662.json` | reference-only | Historical source/provider inventory。現在の GitHub/release truth ではありません。 |
| `docs/reference/open-pr-issue-reconciliation-662.md` | reference-only | Historical reconciliation narrative。現在の authorization ではありません。 |
| `docs/reference/operation-time-policy-reevaluation.{ja,md,zh-CN}.md` | implemented-different-by-design | Rust Core `OperationTimeRequest` の strict fail-closed evaluator を追加します。評価だけを行い、実行/provider 権限付与はしません。 |
| `docs/reference/performance-diagnosis.md` | implemented-different-by-design | Source diagnosis を request-scoped Rust `diagnose` と advisory cost observation に写像し、provider wait/P95/assurance を発明しません。 |
| `docs/reference/pre-release-documentation-alignment.json` | reference-only | Historical generated assessment receipt。target documentation は独自の check と evidence を使います。 |

## Verification

- Ledger は WI-348 の十件（implemented-different-by-design 七件、reference-only 三件）を
  一度ずつ持ち、deferred/migrate-gap は残しません。
- `OperationTimeRequest` は unsupported schema、unknown operation、operation/target/scope
  mismatch、scope/authority 欠落、stale evidence、untrusted input、未分類 impact を
  fail-closed にし、操作を実行しません。
- 英語・簡体字中国語・日本語の文書リンクを揃え、固定表示 label を localize しても
  Contract bytes は変更しません。
- Ledger は pinned target baseline と current count を示し、historical provider/pre-release
  record は `.ai/` や status にコピーしません。
- Rust、documentation/inventory、format、lint、locked workspace verification と、reviewed
  merge 前の installed Runtime の人向け Outcome を確認します。
