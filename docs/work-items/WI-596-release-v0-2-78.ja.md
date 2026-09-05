---
author: AI Cockpit maintainers
title: "WI-596 — v0.2.78 release と object-adopter recovery handoff"
description: "archived Work Item recovery compatibility 修正を含む Runtime を公開し、public artifact を検証します。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-596-release-v0-2-78
lastVerifiedBy: WI-596-release-v0-2-78
terminalArchive: .ai/work-items/archive/WI-596-release-v0-2-78.contract.json
terminalVerification: .ai/evidence/WI-596-release-v0-2-78.verification.json
terminalFinalization: .ai/decisions/WI-596-release-v0-2-78.finalize.json
terminalDecision: .ai/decisions/WI-596-release-v0-2-78.close.json
---

[English](WI-596-release-v0-2-78.md) · [简体中文](WI-596-release-v0-2-78.zh-CN.md)

# WI-596 — v0.2.78 release と object-adopter recovery handoff

## 目的

review 済みで同期された default branch から v0.2.78 を公開します。この patch Release は
既にレビュー済みの Contract amendment predecessor-close recovery 修正を公開し、失敗した
v0.2.77 tag を immutable な履歴として保持し、object repository 向けに再現可能な
public-artifact acceptance handoff を提供します。

## 境界

この Work Item は package version metadata と Release 文書だけを変更します。Runtime source
behavior、object repository、global Agent/MCP 設定、historical evidence bytes、reference-source
実装は範囲外です。public adopter と N-1 acceptance は公開後の evidence であり、source checkout
や workspace build ではなく download 済みの immutable artifact だけを使用します。

## 受入れ

1. Cargo metadata と lockfile は v0.2.78 になり、v0.2.77 は失敗した未公開履歴として保持し、再 tag や installation baseline 化をしません。
2. Release policy check が annotated tag、5 target artifact、checksum、SBOM/provenance、Runtime identity を一つの reviewed commit に bind します。
3. 公開後の adopter/N-1 harness は v0.2.78 artifact だけを使い、forbidden root isolation と成功/失敗時の temporary-run cleanup を証明します。
4. object repository は変更せず、公開後に正確な compatibility/recovery/revalidation command をチームへ渡します。
5. Release または adopter が失敗しても公開済みの事実を保持し、failure receipt を記録します。失敗 tag や historical evidence は書き換えません。
6. 英語・簡体中国語・日本語の release/versioning 文書は current public baseline と installation command で一致します。

## 検証

Contract に列挙した locked workspace、documentation、parity、release policy、staged acceptance、公開後 public-artifact check を実行します。reviewed PR check、v0.2.78 Release、adopter/N-1 receipt 保存、正確な branch/worktree cleanup を確認してから lifecycle を完了します。
