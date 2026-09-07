---
author: AI Cockpit maintainers
title: "WI-632 - WI-631 ドキュメント昇格"
description: "検証済み WI-631 の終端状態を三言語の読者向けドキュメントへ反映する。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-632-doc-promotion-wi631
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-632-doc-promotion-wi631
---

# WI-632 - WI-631 ドキュメント昇格

## 意図と境界

クローズ済み WI-631 のリファレンス再ベースラインを英語・中国語・日本語の
読者向けドキュメントへ反映する。本 Work Item は宣言した 9 文書の投影だけを変更し、
Runtime、参照ソースのバイト列、対象リポジトリは変更しない。

## 受入れ

- 三言語の WI-631 終端状態と証跡リンクが最新である。
- 検証前に三言語の parity 台帳へ本 Work Item を登録する。
- ドキュメントとクローズ済み Work Item の昇格チェックが通過する。

## 検証

```text
bash tests/ci/recovery_gate_acceptance.sh
python3 tests/docs/promote_closed_work_item.py --repo . --check-all
```

参照：[English](WI-632-doc-promotion-wi631.md) · [中文](WI-632-doc-promotion-wi631.zh-CN.md)。
