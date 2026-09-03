---
author: AI Cockpit maintainers
title: "WI-542 — WI-541 終端ドキュメント promotion"
description: "完了した WI-541 の証拠に基づき文書を昇格し、この限定投影を登録する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-542-doc-promotion
lastVerifiedBy: WI-542-doc-promotion
---

[English](WI-542-doc-promotion.md) · [简体中文](WI-542-doc-promotion.zh-CN.md)

## 目的

不変の WI-541 close 証拠に三言語のリリースページと parity 行を同期し、
このドキュメント Work Item を登録する。

## 範囲と境界

- WI-541 の三言語リーダーページと三つの reference parity 台帳。
- この Work Item 自身の三言語リーダーページと parity 登録。
- Runtime 動作、生成された `.ai` レコード、リリース成果物、対象リポジトリは
  対象外。

## 受入れ

- WI-541 の投影が終端証拠を持ち、`implemented` になる。
- WI-542 のページと parity 行が言語リンクと意味的同等性を保つ。
- ドキュメント受入れ、状態整合性、parity 完全性、close 後 promotion 検査が通る。

## 証拠境界

Promotion は読者向け投影だけを変更する。不変の Contract、verification、
finalization、close レコードは Runtime が所有する。
