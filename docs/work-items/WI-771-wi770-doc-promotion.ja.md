---
author: AI Cockpit maintainers
title: "WI-771 — WI-770 terminal documentation promotion"
description: "検証済みでクローズされた WI-770 のドキュメント投影を終端状態へ昇格する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-771-wi770-doc-promotion
lastVerifiedBy: WI-771-wi770-doc-promotion
---

[English](WI-771-wi770-doc-promotion.md) · [简体中文](WI-771-wi770-doc-promotion.zh-CN.md)

# WI-771 — WI-770 terminal documentation promotion

検証済みでクローズされた `WI-770-performance-terminal-docs-recovery` の
ドキュメントと reference-parity 投影を終端状態へ昇格し、過去の evidence や
Runtime は変更しません。

対象は WI-770 の六つのドキュメント投影と本 Work Item のガバナンス文書だけであり、
性能実装、リリース動作、バージョンメタデータ、WI-770 の過去の
archive/evidence/finalization/close バイト列は対象外とします。

closed Work Item promotion helper の `--check-all` 成功後に終端リンクを投影します。
