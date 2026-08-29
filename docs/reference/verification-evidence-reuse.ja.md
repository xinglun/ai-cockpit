---
author: AI Cockpit maintainers
title: Verification evidence reuse の判断
description: 安全で測定可能な Verification 再利用の判断境界。
audience: [adopter, maintainer, reviewer]
status: implemented
authority: translation
canonical: docs/reference/verification-evidence-reuse.md
lastVerifiedBy: WI-379-reference-documentation-batch-18
---

# Verification evidence reuse の判断

[English](verification-evidence-reuse.md) · [简体中文](verification-evidence-reuse.zh-CN.md) · [日本語](verification-evidence-reuse.ja.md)

Evidence classifier は receipt を fresh、stale、unknown に分類し、planner が結果を消費し、
bounded adapter が必要な execution を行います。fresh receipt で skip できるのは allowlist
された非保護 node だけです。unknown/stale は通常どおり再実行し、security、scope、
governance、coverage、source-bound の protected node は再利用で skip しません。

## 必須 binding

base/head revision、正規化 changed paths、command と digest、environment/toolchain、policy、
stage、runner、repository/Work Item identity、output receipt digest の完全一致が必要です。
どれかが変われば候補は無効になり、Runtime は時間、cache label、provider result から安全性を
推測しません。

## コストと限界

adapter は planned、executed、reused、stale、unknown、protected の call count を報告します。
無関係な変更で実際の call が減り、protected call が変わらないことだけが最適化の観測です。
local run から provider/human wait、P95、assurance 向上を主張しません。source の Python/Make
orchestration と JSONL record は reference material のまま、Rust は typed repository-bound
receipt で同じ trust boundary を保ちます。
