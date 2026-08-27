---
author: AI Cockpit maintainers
workItemId: WI-333-comprehension-validation
title: "WI-333 — Reference comprehension-validation protocol and participant records"
description: "Compare the pinned comprehension-validation study files and record a non-transferable target boundary."
audience:
  - adopter
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-333-comprehension-validation
capabilityClaims:
  - reference_parity
---

# WI-333 — Reference comprehension-validation protocol and participant records

## Intent

Compare the pinned reference files one by one and record a safe, auditable
classification. This Work Item establishes a target boundary; it does not run a
participant study and does not transfer human-subject evidence.

## Scope and decision

All twelve pinned paths below are `reference-only`. They remain owned by the
reference repository because their procedures, pseudonyms, revisions, answers,
sample counts and bounded study conclusions are not portable to this repository.
The target counterparts describe its own reader route, Agent workflow, Contract,
Outcome and Runtime evidence boundaries. No source response or result bytes are
copied, and no target comprehension, release, safety, security or enterprise
claim is inferred.

| Pinned source path | Target counterpart(s) | Decision |
| --- | --- | --- |
| `docs/reference/comprehension-validation-protocol.md` | `docs/README.md`; `docs/reference/agent-workflow.md`; `docs/reference/outcome-report.md` | `reference-only`; external eligibility, consent and interview protocol |
| `docs/reference/comprehension-validation-protocol.zh-CN.md` | localized README, Agent workflow and Outcome report | `reference-only`; no target participant study |
| `docs/reference/comprehension-validation-protocol.ja.md` | localized README, Agent workflow and Outcome report | `reference-only`; source ethics are not Runtime policy |
| `docs/reference/comprehension-validation-response.schema.json` | `.ai/README.md`; `docs/reference/outcome-report.md` | `reference-only`; not the Runtime Contract/evidence schema |
| `docs/reference/comprehension-validation-responses/peter_01.en.json` | English README and human-benefit report | `reference-only`; historical source response |
| `docs/reference/comprehension-validation-responses/peter_02.en.json` | English README and human-benefit report | `reference-only`; no participant data import |
| `docs/reference/comprehension-validation-responses/tanaka_01.ja.json` | Japanese README and human-benefit report | `reference-only`; not adopter evidence |
| `docs/reference/comprehension-validation-responses/tanaka_02.ja.json` | Japanese README and human-benefit report | `reference-only`; source revision-bound |
| `docs/reference/comprehension-validation-responses/xiaoli_01.zh-CN.json` | Chinese README and human-benefit report | `reference-only`; no target native-language score |
| `docs/reference/comprehension-validation-responses/xiaoli_02.zh-CN.json` | Chinese README and human-benefit report | `reference-only`; raw text not copied |
| `docs/reference/comprehension-validation-results.json` | tri-language human-benefit report and comparison | `reference-only`; source sample/result remains revision-bound |
| `docs/reference/comprehension-validation-results.md` | English human-benefit and Outcome reports | `reference-only`; source limitations are not target evidence |

## Acceptance

- Inventory contains exactly twelve WI-333 records, all `reference-only`, with non-empty counterparts and reasons.
- No WI-333 record is deferred or marked for migration, and no participant response/result is copied into target evidence.
- Tri-language comparison, parity, and Work Item documentation state the same boundary.
- Documentation and inventory checks pass; installed Runtime lifecycle produces current evidence and the reviewed PR is merged, closed, and cleaned.

## Object/adopter boundary

An adopter repository inherits the target's documentation route, Contract,
evidence and Agent workflow, not another repository's human-subject records. A
future study needs its own human-owned Contract covering consent, retention,
privacy and evidence before any result can be claimed.

Language versions: [简体中文](WI-333-comprehension-validation.zh-CN.md) · [日本語](WI-333-comprehension-validation.ja.md)
