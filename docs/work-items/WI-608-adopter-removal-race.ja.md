---
title: "WI-608 — adopter checkout の競合耐性クリーンアップ"
description: "公開リリース受入れの一時 checkout を限定的に再試行し、失敗時は fail-closed に記録する。"
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
lastVerifiedBy: WI-608-adopter-removal-race
terminalArchive: .ai/work-items/archive/WI-608-adopter-removal-race.contract.json
terminalVerification: .ai/evidence/WI-608-adopter-removal-race.verification.json
terminalFinalization: .ai/decisions/WI-608-adopter-removal-race.finalize.json
terminalDecision: .ai/decisions/WI-608-adopter-removal-race.close.json
workItemId: WI-608-adopter-removal-race
---

[English](WI-608-adopter-removal-race.md) · [简体中文](WI-608-adopter-removal-race.zh-CN.md)

# WI-608 — adopter checkout の競合耐性クリーンアップ

## 目的

リリース adopter ハーネスは正確な一時 checkout だけを削除する。Git の一時的な
maintenance 競合には限定回数の再試行で対応し、クリーンアップ失敗は受入れ証跡を
保持したまま明示的に失敗させる。

## 境界

対象は staged/public adopter、N-1 受入れスクリプト、その回帰テスト、およびリリース
ワークフローの認証受け渡しである。Runtime のガバナンス意味論、object repository、
グローバル Agent/MCP 設定は変更しない。

## 証跡

- Archive: `.ai/work-items/archive/WI-608-adopter-removal-race.archive.json`
- Verification: `.ai/evidence/WI-608-adopter-removal-race.verification.json`
- Finalization: `.ai/decisions/WI-608-adopter-removal-race.finalize.json`
- Close: `.ai/decisions/WI-608-adopter-removal-race.close.json`
