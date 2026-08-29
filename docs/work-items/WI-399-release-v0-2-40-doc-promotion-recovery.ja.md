---
author: AI Cockpit maintainers
title: "WI-399 — v0.2.40 ドキュメント昇格リカバリ"
description: "専用 worktree で WI-398 の配送を復旧し、監査可能な release baseline を保持する。"
workItemId: WI-399-release-v0-2-40-doc-promotion-recovery
audience: [maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-399-release-v0-2-40-doc-promotion-recovery
terminalArchive: .ai/work-items/archive/WI-399-release-v0-2-40-doc-promotion-recovery.contract.json
terminalVerification: .ai/evidence/WI-399-release-v0-2-40-doc-promotion-recovery.verification.json
terminalFinalization: .ai/decisions/WI-399-release-v0-2-40-doc-promotion-recovery.finalize.json
terminalDecision: .ai/decisions/WI-399-release-v0-2-40-doc-promotion-recovery.close.json
capabilityClaims: [documentation_governance, release_distribution]
---

# WI-399 — v0.2.40 ドキュメント昇格リカバリ

[English](WI-399-release-v0-2-40-doc-promotion-recovery.md) · [简体中文](WI-399-release-v0-2-40-doc-promotion-recovery.zh-CN.md)

## Intent

WI-398 の finalization が main worktree 上で正しく拒否されたため、専用
worktree で配送を復旧する。WI-398 の不変 archive と recovery decision は書き換えない。

## Boundary

対象は recovery decision、三言語 WI-399 文書、reference-parity 登録だけである。
Runtime semantics、release 実装、公開 adopter acceptance、履歴 evidence bytes は対象外とする。

## Verification and delivery

archive 前にドキュメント受入れ、repository governance checks、locked workspace
全量テストを通過させる。レビュー済み PR の merge 後に branch と専用 worktree を削除し、
正確な cleanup と successor の close を記録する。
