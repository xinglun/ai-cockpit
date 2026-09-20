---
author: AI Cockpit maintainers
title: "WI-954 — ドキュメント投影の復旧"
description: "リリース検証を繰り返さずに、リリース後のドキュメント投影経路を復旧する。"
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:xinglun
workItemId: WI-954-start-verification-declaration
lastVerifiedBy: WI-954-start-verification-declaration
---

[English](WI-954-start-verification-declaration.md) · [简体中文](WI-954-start-verification-declaration.zh-CN.md)

# WI-954 — ドキュメント投影の復旧

## 目的

WI-953 の置換後に停止したリリース後ドキュメント投影経路を、リリース検証を繰り返さず、Runtime の動作も変えずに復旧する。

## 境界

この Work Item は三言語の人間向け文書投影と parity 行だけを変更する。release asset、完了済み検証証跡、object repository は対象外である。

## 受入れ

- 投影 helper が Cargo test を起動せず安定状態に到達する。
- 三言語ページと parity 行が一致する。
