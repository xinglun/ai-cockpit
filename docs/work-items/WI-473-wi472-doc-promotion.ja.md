---
author: AI Cockpit maintainers
title: "WI-473 — WI-472 終端ドキュメント昇格"
description: "release 前に terminal Work Item と parity projection を完全に保つ。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-473-wi472-doc-promotion
status: implemented
authority: authorized
lastVerifiedBy: WI-473-wi472-doc-promotion
terminalArchive: .ai/work-items/archive/WI-473-wi472-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-473-wi472-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-473-wi472-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-473-wi472-doc-promotion.close.json
---

# WI-473 — WI-472 終端ドキュメント昇格

## Intent と境界

検証済み WI-472 の lifecycle を reader-facing documentation に昇格し、
recovery と現在の Work Item の parity 登録を監査可能にします。本 Work Item
は documentation projection だけを変更し、immutable `.ai` records、Runtime
code、CI、release artifact、object repository は範囲外です。

## Scope

- close 後に WI-472 の英語・簡体字中国語・日本語ページを昇格する。
- 3 つの parity ledger で WI-471 の authoritative hashed recovery receipt を保持する。
- archive/close 前にこの Work Item と terminal path を登録する。

## Acceptance

1. 3 言語の WI-472 page と parity row が terminal receipt を bind する。
2. 3 言語の WI-473 page と pre-archive parity row が governance integrity gate を通る。
3. documentation と reference-inventory check が clean branch で通る。
4. immutable governance bytes と object repository を変更しない。

## Verification

- `cargo test --locked --workspace`
- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/docs/documentation_acceptance.sh`

## Recovery boundary

projection が不足した場合は immutable records を保持し、explicit amendment と
revalidation により現在の documentation Work Item を修復する。
