---
author: AI Cockpit maintainers
title: "WI-374 — v0.2.39 release と exact verification reuse の受入れ"
description: "復旧 parity projection を修正し、dynamic verification-reuse Runtime を公開して隔離 repository で受け入れる。"
workItemId: WI-374-release-v0-2-39
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-374-release-v0-2-39
terminalArchive: .ai/work-items/archive/WI-374-release-v0-2-39.contract.json
terminalVerification: .ai/evidence/WI-374-release-v0-2-39.verification.json
terminalFinalization: .ai/decisions/WI-374-release-v0-2-39.finalize.json
terminalDecision: .ai/decisions/WI-374-release-v0-2-39.close.json
capabilityClaims: [release_distribution, verification_performance, adopter_acceptance]
---

# WI-374 — v0.2.39 release と exact verification reuse の受入れ

[English](WI-374-release-v0-2-39.md) · [简体中文](WI-374-release-v0-2-39.zh-CN.md)

## Intent

レビュー済みで同期された `main` から v0.2.39 を準備し、identity-bound な dynamic exact verification reuse を公開可能にする。公開前に WI-370 と WI-371 の復旧 receipt の parity projection を修正する。公開 artifact と adopter 受入れは immutable tag 後の successor WI-376 に明示的に引き継ぐ。

## Scope と境界

- 三言語の Cargo metadata、lockfile、versioning、release、distribution 文書を v0.2.39 に揃える。
- 三つの parity ledger で digest-suffixed な authoritative recovery receipt を参照し、predecessor evidence は書き換えない。
- immutable tag 前に必要な strict release policy と staged check を実行し、公開前に public artifact を主張しない。
- post-release の引き継ぎを残し、successor WI-376 が公開 artifact だけをダウンロードして本 repository と新しい隔離 adopter に導入する。

Runtime semantics、historical evidence の書換え、global Agent/MCP 設定、source-build fallback、第二技術 stack adopter は本 Work Item の範囲外である。

## Acceptance

1. Cargo metadata と lockfile が v0.2.39 で一致する。
2. 復旧 parity 行が authoritative recovery receipt を参照し、strict documentation/governance gate が通る。
3. 公開 Release asset、checksum、SBOM、Formula、provenance の受入れは successor WI-376 に延期し、本 WI では主張しない。
4. 公開 binary identity と fallback 不使用の受入れは successor WI-376 に延期する。
5. exact reuse と新規 adopter の受入れは successor WI-376 に延期する。
6. isolation、cleanup、lifecycle、失敗時の Release truth 維持の受入れは successor WI-376 に延期する。
7. レビュー済み merge、finalization、close、default branch 同期、正確な branch/worktree cleanup 後に `ready_on_base` となる。

## Verification boundary

公開前は strict repository gate manifest と staged check を使う。公開後受入れは successor WI-376 の境界であり、immutable な v0.2.39 artifact だけをダウンロードし、tag、archive digest、binary digest、platform、source を記録する。最適化は exact-match reuse のみであり、初回または無効化された検証は必ず実行する。測定した効果をそれらの経路へ外挿しない。
