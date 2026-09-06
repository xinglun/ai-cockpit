---
author: AI Cockpit maintainers
title: "WI-624 — WI-622 ドキュメント昇格リカバリ"
description: "最初のドキュメント昇格が parity gate で拒否された際の限定的な投影漏れを修正する。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-624-wi622-doc-promotion-recovery
status: in_progress
authority: canonical
lastVerifiedBy: WI-624-wi622-doc-promotion-recovery
---

# WI-624 — WI-622 ドキュメント昇格リカバリ

## Intent

WI-622 の限定的なドキュメント昇格を完了し、この recovery Work Item 自身の
英語・中国語・日本語ページと parity ledger の記録も含めます。CI で見つかった
自己投影の漏れを修正しますが、過去の Contract、evidence、archive、receipt、
decision bytes は変更しません。

## Boundary

対象は WI-622 の三言語ページ、この Work Item の三言語ページ、三つの reference
parity ledger だけです。Runtime code、release/adopter acceptance、対象 repository、
global Agent/MCP configuration、生成済みの履歴記録は対象外です。

## Acceptance

- WI-622 の三言語 reader-facing status と terminal evidence link を `Implemented`
  に昇格する。
- 検証前に本 recovery Work Item の三言語ページを作成し、三つの parity ledger に登録する。
- closed Work Item promotion、parity、documentation checks が成功する。

## Verification

```text
bash tests/docs/parity_status_check.sh
bash tests/docs/documentation_acceptance.sh
```

参照：[English](WI-624-wi622-doc-promotion-recovery.md) ·
[中文](WI-624-wi622-doc-promotion-recovery.zh-CN.md)。
