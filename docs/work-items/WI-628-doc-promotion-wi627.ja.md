---
author: AI Cockpit maintainers
title: "WI-628 — WI-627 ドキュメント昇格"
description: "検証済み WI-627 の終端状態を三言語の reader documentation に反映する。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-628-doc-promotion-wi627
status: in_progress
authority: canonical
lastVerifiedBy: WI-628-doc-promotion-wi627
---

# WI-628 — WI-627 ドキュメント昇格

## Intent

close 済みの WI-627 reference rebaseline を英語・中国語・日本語の reader
documentation に反映します。Contract、evidence、archive、finalization、decision
bytes は変更しません。

## Boundary

対象は WI-627 の三言語ページ、この Work Item の三言語ページ、三つの reference
parity ledger です。Runtime code、tests、CI、reference ledger、生成された governance
records、global Agent/MCP configuration は対象外です。

## Acceptance

- WI-627 の三言語 reader-facing status と terminal evidence link が最新である。
- 検証前に本 Work Item の三言語ページを作成し、三つの parity ledger に登録する。
- documentation checks がすべて成功する。

## Verification

```text
python3 tests/docs/promote_closed_work_item.py --check-all
python3 tests/docs/reference_comparison_metadata_test.py
python3 tests/conformance/reference_inventory_docs_test.py
bash tests/docs/documentation_acceptance.sh
```

参照：[English](WI-628-doc-promotion-wi627.md) ·
[中文](WI-628-doc-promotion-wi627.zh-CN.md)。
