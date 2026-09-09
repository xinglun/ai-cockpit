---
author: AI Cockpit maintainers
title: "WI-766 — WI-765 terminal documentation promotion"
description: "検証済みでクローズされた WI-765 のドキュメント投影を終端状態へ昇格する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human:repository-owner
workItemId: WI-766-wi765-doc-promotion
lastVerifiedBy: WI-766-wi765-doc-promotion
terminalArchive: .ai/work-items/archive/WI-766-wi765-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-766-wi765-doc-promotion.verification.json
terminalFinalization: .ai/decisions/WI-766-wi765-doc-promotion.finalize.f46dd4dba8e4a0017793479963bf476447609f71e73f8b0f85e7502312775bad.json
terminalDecision: .ai/decisions/WI-766-wi765-doc-promotion.close.json
---

[English](WI-766-wi765-doc-promotion.md) · [简体中文](WI-766-wi765-doc-promotion.zh-CN.md)

# WI-766 — WI-765 terminal documentation promotion

検証済みでクローズされた `WI-765-wi764-doc-promotion` のドキュメントと
reference-parity 投影を終端状態へ昇格し、過去の evidence は変更しない。

対象は WI-765 の六つのドキュメント投影と本 Work Item のガバナンス文書だけであり、
Runtime、リリース動作、バージョンメタデータ、WI-765 の過去の archive/evidence/
finalization/cleanup/close バイト列は対象外とする。

closed Work Item promotion helper の `--check-all` 成功後に終端リンクを投影する。
