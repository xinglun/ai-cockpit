---
author: AI Cockpit メンテナー
title: "WI-844 — lifecycle resource reconciliation"
description: "Work Item の lifecycle と正確な resource を収束させながら、historical successor binding を保持する。"
audience: [maintainer, reviewer]
status: in_progress
authority: authorized
workItemId: WI-844-lifecycle-resource-reconciliation
---

[English](WI-844-lifecycle-resource-reconciliation.md) · [简体中文](WI-844-lifecycle-resource-reconciliation.zh-CN.md)

# WI-844 — lifecycle resource reconciliation

## Intent と boundary

この Work Item は reviewed merge 後に残る Work Item lifecycle、remote branch、local worktree を
収束させます。historical Contract、Summary、Outcome、event、evidence の bytes は保持し、未対応または
欠落した historical evidence は明示的に分類します。正確に一致する clean resource で、reviewed merge と
terminal lifecycle が確認できた場合だけ削除します。release tag、公開 Release、product behavior、無関係な
source change は対象外です。

## Recovery boundary

同じ範囲の lifecycle defect はこの Work Item を amend して再検証します。scope、authority、base が異なる
場合、独立した変更、安全でない in-scope repair、immutable な failed delivery、または明示的な human direction
の場合だけ successor を使います。既に binding された successor は immutable な checkpoint Contract digest
だけを使って amended predecessor を supersede できます。未 binding または競合する successor は fail-closed です。

## Acceptance

- すべての active Work Item に evidence に基づく disposition（closed、具体的理由付き blocked、または current implementation として保持）がある。
- historical record は byte-for-byte で保持し、field の捏造や current snapshot に対する再検証で完了扱いにしない。
- lifecycle close 後に reviewed merge が確認できる正確な clean branch/worktree だけを削除し、それ以外は理由付きで保持する。
- predecessor Contract の additive amendment で、strict に binding された successor の checkpoint digest を無効にせず、historical product verification も再実行しない。

## Verification

- `cargo test --locked -p cockpit-repository --test recovery_decision`
- `cargo fmt --all -- --check`
- `git diff --check`
