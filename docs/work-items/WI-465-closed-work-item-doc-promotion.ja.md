---
author: AI Cockpit maintainers
title: "WI-465 — closed Work Item ドキュメント昇格"
workItemId: WI-465-closed-work-item-doc-promotion
description: "immutable な記録を書き換えず、closed Work Item の evidence を読者向け文書へ昇格します。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-465-closed-work-item-doc-promotion
terminalArchive: .ai/work-items/archive/WI-465-closed-work-item-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-465-closed-work-item-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-465-closed-work-item-doc-promotion.finalize.json
terminalDecision: .ai/decisions/WI-465-closed-work-item-doc-promotion.close.json
---

# WI-465 — closed Work Item ドキュメント昇格

この Work Item は WI-464 recovery retry で判明した close 後の文書 projection
不足を修正します。Runtime の immutable な archive、verification、finalization、
close evidence からのみ昇格し、それらの記録は書き換えません。

[English](WI-465-closed-work-item-doc-promotion.md) · [简体中文](WI-465-closed-work-item-doc-promotion.zh-CN.md)

## Scope

- WI-464 retry の三言語ページと parity 行を昇格する。
- close 後の同じ昇格チェックが新しい文書 debt を作らないよう、この Work Item
  自身の三言語ページと parity 登録を維持する。
- canonical gate manifest の closed Work Item チェックと stale projection 回帰
  テストを維持する。
- Runtime 挙動、参照 source bytes、object repository、immutable `.ai` evidence は
  範囲外とする。

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `bash tests/docs/promote_closed_work_item_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
- `cargo test --locked --workspace`

このページの terminal fields は、review 済み merge、archive、finalization、close
境界を通過した後の promotion pass だけが書き込みます。
