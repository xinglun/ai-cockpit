---
author: AI Cockpit maintainers
title: "WI-476 — WI-475 terminal documentation promotion"
description: "不変レコードを書き換えず、close 済み WI-475 の evidence を reader-facing projection に昇格します。"
audience:
  - maintainer
  - reviewer
  - adopter
workItemId: WI-476-wi475-doc-promotion
status: implemented
authority: authorized
lastVerifiedBy: WI-476-wi475-doc-promotion
terminalArchive: .ai/work-items/archive/WI-476-wi475-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-476-wi475-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-476-wi475-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-476-wi475-doc-promotion.close.json
---

# WI-476 — WI-475 terminal documentation promotion

## Intent と boundary

この bounded Work Item は、verified/closed の WI-475 lifecycle を三言語の
Work Item と reference-parity projection に昇格します。不変 Runtime evidence、
reference inventory、Runtime code、object repository は変更しません。

[English](WI-476-wi475-doc-promotion.md) · [简体中文](WI-476-wi475-doc-promotion.zh-CN.md)

## Scope

- 三言語の WI-475 Work Item page と parity ledger に archive、verification、
  finalization、close record を結び付けます。
- archive 前に本 Work Item 自身の三言語 page と parity row を登録し、verified close 後にだけ昇格します。
- closed Work Item promotion check を再現可能にし、過去の evidence bytes をそのまま保持します。

## Out of scope

Runtime/Core 実装、reference inventory 分類、release/adopter script、object repository、
global Agent/MCP configuration。

## Acceptance

1. `promote_closed_work_item.py --repo <repo> --work-item WI-475-reference-file-comparison-batch-25 --check` が成功すること。
2. WI-475 の六つの projection file が正確な archive、verification、finalization、close evidence path を参照すること。
3. 本 Work Item に三言語 page と pre-archive parity row があり、close 後の promotion が決定的であること。
4. Contract、archive、verification、finalization、close、reference inventory の bytes を書き換えないこと。
5. 英語・簡体字中国語・日本語の page が意味的に同等で、Contract の authored language を保持すること。

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item WI-475-reference-file-comparison-batch-25 --check`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/getting_started_semantic.sh`
- `bash tests/docs/parity_status_check.sh`
- `bash tests/ci/governance_integrity_gate_test.sh`
- `python3 tests/ci/repository_gate_manifest_test.py`
- `cargo test --locked --workspace`

この page の terminal field は、reviewed merge、archive、finalization、close 完了後にのみ昇格されます。
