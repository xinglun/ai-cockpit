---
author: AI Cockpit maintainers
title: "WI-481 — WI-480 terminal documentation promotion"
description: "不変 evidence を書き換えず、WI-480 の terminal documentation projection を昇格します。"
audience:
  - maintainer
  - reviewer
  - adopter
workItemId: WI-481-wi480-doc-promotion
status: in_progress
authority: authorized
lastVerifiedBy: WI-481-wi480-doc-promotion
---

# WI-481 — WI-480 terminal documentation promotion

この bounded Work Item は、verified/closed の WI-480 lifecycle を三言語の
Work Item と reference-parity projection に昇格します。不変 Runtime evidence、
archive record、reference inventory は変更しません。

[English](WI-481-wi480-doc-promotion.md) · [简体中文](WI-481-wi480-doc-promotion.zh-CN.md)

## Scope

- repository helper で WI-480 の六つの documentation projection を昇格します。
- 正確な terminal record に結び付け、決定的な promotion を維持します。
- archive 前に本 Work Item の page と parity row を登録します。

## Out of scope

Runtime/Core 実装、release/adopter artifact、これらの projection を超える reference-source parity、
不変 governance bytes。

## Acceptance

1. `promote_closed_work_item.py --repo <repo> --work-item WI-480-finalization-context-recovery --check` が成功すること。
2. merge 後に `promote_closed_work_item.py --repo <repo> --check-all` が stale projection を報告しないこと。
3. Contract、Summary、Outcome、Evidence、Finalization、Close、Recovery、reference inventory の bytes を書き換えないこと。

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-480-finalization-context-recovery --check`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/getting_started_semantic.sh`
- `bash tests/docs/parity_status_check.sh`
- `bash tests/ci/governance_integrity_gate_test.sh`
- `python3 tests/ci/repository_gate_manifest_test.py`
- `cargo test --locked --workspace`
