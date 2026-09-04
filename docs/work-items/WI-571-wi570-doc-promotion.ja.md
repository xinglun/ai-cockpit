---
author: AI Cockpit maintainers
title: "WI-571 — WI-570 の終端ドキュメント昇格"
description: "immutable な governance record を書き換えず、close 済み WI-570 の documentation projection を昇格する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-571-wi570-doc-promotion
lastVerifiedBy: WI-571-wi570-doc-promotion
---

[English](WI-571-wi570-doc-promotion.md) · [简体中文](WI-571-wi570-doc-promotion.zh-CN.md)

# WI-571 — WI-570 の終端ドキュメント昇格

## 目的

WI-570 の verified-close documentation page を昇格し、この documentation projection を
三言語 parity matrix に登録する。immutable な governance record は変更しない。

## 範囲と境界

対象は三言語の WI-570 page、三言語の WI-571 page、および三言語の reference-parity page
である。Runtime behavior、release artifact、対象 repository、global Agent/MCP 設定、過去の
governance bytes は対象外とする。

## 受入れ

- 三言語の WI-570 page が `implemented` となり、archive、verification、finalization、close
  evidence を参照すること。
- 三言語 parity page が WI-570 を実装済みとし、evidence path 付きで WI-571 の終端 projection
  を登録すること。
- documentation、parity、promotion、diff check が immutable governance record を書き換えずに
  pass すること。
- WI-571 に対応する英語、簡体字中国語、日本語 page があること。

## 検証

- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`
