---
author: AI Cockpit maintainers
workItemId: WI-886-verification-isolation-v0-2-95
title: リリース受入れの隔離を修復し v0.2.95 を公開する
description: 不変の v0.2.94 受入れ失敗後、明示的な検証ターゲット隔離を維持し、候補とダウンロード成果物の受入れを証明する。
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorization
predecessorWorkItemId: WI-883-release-v0-2-94-current-main
lastVerifiedBy: WI-886-verification-isolation-v0-2-95
terminalArchive: .ai/work-items/archive/WI-886-verification-isolation-v0-2-95.contract.json
terminalVerification: .ai/evidence/WI-886-verification-isolation-v0-2-95.verification.json
terminalFinalization: .ai/decisions/WI-886-verification-isolation-v0-2-95.finalize.json
terminalDecision: .ai/decisions/WI-886-verification-isolation-v0-2-95.close.json
---

# WI-886 — リリース受入れの隔離を修復し v0.2.95 を公開する

この successor は不変の v0.2.94 タグと段階受入れの失敗証拠を保持する。呼び出し元が指定した隔離 `CARGO_TARGET_DIR` を尊重するよう検証環境境界を修復し、文書化された HOME フォールバックと `CARGO_INCREMENTAL=0` は維持する。v0.2.95 の公開は、レビュー済みマージ、候補受入れ、ダウンロード成果物のインストール/アップグレード受入れの後に行う最後の手順である。

隔離、クリーンアップ、検証、リリース識別子、残る未知を別々に報告し、候補やソースチェックアウトを公開成功と推測しない。
