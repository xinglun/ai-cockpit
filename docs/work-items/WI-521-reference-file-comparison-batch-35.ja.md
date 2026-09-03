---
author: AI Cockpit maintainers
title: "WI-521 — reference guard と adoption check の batch 35"
description: "次の bounded reference script を一つずつ比較し、source tooling を copy せず Rust boundary を記録します。"
audience:
  - maintainer
  - reviewer
  - adopter
status: implemented
authority: canonical
workItemId: WI-521-reference-file-comparison-batch-35
lastVerifiedBy: WI-521-reference-file-comparison-batch-35
terminalArchive: .ai/work-items/archive/WI-521-reference-file-comparison-batch-35.contract.json
terminalVerification: .ai/evidence/WI-521-reference-file-comparison-batch-35.verification.json
terminalFinalization: .ai/decisions/WI-521-reference-file-comparison-batch-35.finalize.3963d731bcacd6a4efd4660409749638c2dcc8fe4bcde3a0bf2e8216fa12e2ae.json
terminalDecision: .ai/decisions/WI-521-reference-file-comparison-batch-35.close.json
---

# WI-521 — reference guard と adoption check の batch 35

## Objective

pinned commit `fde3380f81fea5fd2e288f7a8849f737dc074060` の次の reference file を
一つずつ読み、各 current path の evidence-backed classification を記録します。目的は semantic parity と
object/adopter boundary であり、Python/Make command compatibility ではありません。

## File-level result

| Reference path | Decision |
| --- | --- |
| `scripts/ai_check_adoption_ready.py` | `reference-only`: source 固有の adoption completeness。Rust onboarding と status/doctor fact は external boundary です。 |
| `scripts/ai_check_archive_recovery.py` | `implemented-different-by-design`: append-only archive と predecessor-bound recovery が immutable ownership を守ります。 |
| `scripts/ai_check_backtrack.py` | `implemented-different-by-design`: Rust は test/coverage weakening と input-trust signal を導出します。source の report-only deletion warning は maintenance projection です。 |
| `scripts/ai_check_budget_impact.py` | `implemented-different-by-design`: typed identity-bound performance/cost budget は advisory で、必須 verification を置き換えません。 |
| `scripts/ai_check_capability_claims.py` | `reference-only`: source lexical claim/matrix checker は Runtime authority ではなく、Rust capability truth は observed/repository-bound です。 |
| `scripts/ai_check_coverage_guard.py` | `implemented-different-by-design`: Rust は weakening を検出し Contract の verification evidence を bind します。source association report は adopter policy です。 |
| `scripts/ai_check_dependabot_intake.py` | `not-applicable`: bot event identity と automatic merge は provider 固有です。 |
| `scripts/ai_check_diff_ownership.py` | `reference-only`: Rust lifecycle scope と archive ownership が authority で、source cross-Work-Item preview は copy しません。 |
| `scripts/ai_check_guard_calibration.py` | `implemented-different-by-design`: Rust は Project Profile と明示的 calibration fact を検証します。 |
| `scripts/ai_check_guards.py` | `implemented-different-by-design`: typed Contract、authority、trust、lifecycle、isolation が source YAML manifest を置き換えます。 |
| `tests/test_ai_check_archive_recovery.py` | `implemented-different-by-design`: native archive/finalization test が immutable ownership boundary を検証します。 |
| `tests/test_ai_check_budget_impact.py` | `implemented-different-by-design`: native verification/performance test が typed budget と exact reuse を検証します。 |

retired の `tests/test_ai_check_backtrack.py` は current source file として扱わず、append-only ledger の historical metadata に保持します。

## Acceptance

- selected current path はすべて pinned local checkout から読み、
  `tests/conformance/reference_file_inventory.json` に登録します。
- inventory regression は 12 record の reason、counterpart または boundary を要求し、selected record の deferred 残留を拒否します。
- source Python、Make、YAML guard、provider config、object repository file は copy/変更しません。
- tri-language comparison page は同じ count と semantic boundary を示します。

## Object/adopter inheritance

各 attached project は shared Runtime、明示的な `--repo` context、repository-local Contract/evidence/knowledge、fail-closed lifecycle、human Outcome presentation を継承します。source stack command、Dependabot event、CODEOWNERS/SECURITY value、Python report、sample policy decision は継承しません。adopter/provider fact は明示的 external evidence のままです。

## Verification

Finish 前に machine inventory check と documentation/conformance gate を通過させます。この Work Item は Runtime code や governance decision を追加しません。将来の portable extension は新しい bounded Contract で扱います。
