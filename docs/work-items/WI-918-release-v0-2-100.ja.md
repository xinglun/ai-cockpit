---
author: AI Cockpit maintainers
workItemId: WI-918-release-v0-2-100
title: Outcome 言語、HCI、四方向、Issue #851 完了後の最終 v0.2.100 リリース
description: 前提となるすべての Work Item と Issue が完了した後に、レビュー済み main を公開する。
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-918-release-v0-2-100
terminalArchive: .ai/work-items/archive/WI-918-release-v0-2-100.contract.json
terminalVerification: .ai/evidence/WI-918-release-v0-2-100.verification.json
terminalFinalization: .ai/decisions/WI-918-release-v0-2-100.finalize.json
terminalDecision: .ai/decisions/WI-918-release-v0-2-100.close.json
---

[English](WI-918-release-v0-2-100.md) · [简体中文](WI-918-release-v0-2-100.zh-CN.md)

# WI-918 — Outcome 言語、HCI、四方向、Issue #851 完了後の最終 v0.2.100 リリース

このリリース経路は、Outcome 言語、HCI、四方向の収束、Issue #851、Rust/ツールチェーンの
作業完了後に、レビュー済み main だけを公開し、download artifact を独立検証する。

対象外の object repository は変更しない。直接の証拠がない限り、性能利益と host 表示確認は
未知として保持する。

## 受入れ

- workspace version、lock metadata、現在の英語・簡体字中国語・日本語の release/reference projection が v0.2.100 を一貫して示す。
- 注釈付き v0.2.100 tag は不変で、レビュー済み source commit に結び付く。
- 公開 Release は manifest、checksum、SBOM、archive、attestation、source identity、workflow handoff に結び付く。
- ダウンロード成果物は checksum、隔離 fresh-install、v0.2.99 upgrade、正確な一時 root cleanup を通過し、object repository を変更しない。
- 最終 Outcome は現在の会話言語に従い、性能利益と host 表示確認を未知のまま示す。

## 検証計画

dispatch の前に format、version、documentation、parity、governance、workspace の廉価な check を実行する。
publication handoff を消費し、download artifact の adopter acceptance を行い、正確な provider と
branch/worktree cleanup を検証してから close する。
