---
author: AI Cockpit maintainers
title: "WI-630 — WI-629 ドキュメント昇格"
description: "検証済み WI-629 の終端状態を三言語の利用者向け文書へ反映します。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-630-doc-promotion-wi629
status: in_progress
authority: canonical
lastVerifiedBy: WI-630-doc-promotion-wi629
---

# WI-630 — WI-629 ドキュメント昇格

## 意図と境界

クローズ済み WI-629 の reference rebaseline を英語・中国語・日本語の利用者向け文書へ反映します。
本 Work Item は文書のみを変更し、Runtime コード、参照元の bytes、生成済みガバナンス記録、対象プロジェクトは変更しません。

## 受入れ

- 三言語の WI-629 状態と終端 evidence リンクが最新であること。
- 検証前に本 Work Item が三言語の parity 台帳へ登録されていること。
- 文書およびクローズ済み Work Item 昇格チェックが成功すること。

## 検証

```text
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
bash tests/docs/work_item_status_consistency_test.sh
python3 tests/docs/promote_closed_work_item.py --check-all
```

参照：[English](WI-630-doc-promotion-wi629.md) · [中文](WI-630-doc-promotion-wi629.zh-CN.md)。
