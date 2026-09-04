---
author: AI Cockpit maintainers
title: "WI-564 — WI-563 終端ドキュメント昇格"
description: "WI-563 を昇格し、このドキュメント昇格 Work Item 自身を三言語の governance projection に登録する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-564-doc-promotion-wi563
lastVerifiedBy: WI-564-doc-promotion-wi563
---

[English](WI-564-doc-promotion-wi563.md) · [简体中文](WI-564-doc-promotion-wi563.zh-CN.md)

# WI-564 — WI-563 終端ドキュメント昇格

## 目的

検証済み close の WI-563 ドキュメント projection を昇格し、この昇格
Work Item 自身も登録して、documentation governance gate が当 cycle の
すべての Work Item を監査できるようにする。

## 範囲と境界

範囲は WI-563 の三言語 page、WI-564 の三言語 page、および対応する
三つの reference-parity page に限定する。WI-563 の terminal link は
Runtime promotion helper が生成し、WI-564 page は bounded self-projection
を記録するため、この Work Item が検証・close されるまでは in progress
のままとする。

Runtime behavior、object repository、local reference checkout、release
artifact、global Agent/MCP configuration、および immutable な
Contract/evidence/decision/archive bytes は対象外である。

## 受入条件

- WI-563 page が archive、verification、finalization、close の link 付き
  Implemented に昇格される。
- WI-564 page が範囲を説明し、三つの parity page から参照される。
- 三つの parity page が両 Work Item の status と evidence path を一致させ、
  documentation/inventory/governance gate が通過する。
- predecessor の Contract、evidence、decision、archive bytes を書き換えない。

## 検証

- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/conformance/reference_file_inventory_test.sh`
- `git diff --check`
