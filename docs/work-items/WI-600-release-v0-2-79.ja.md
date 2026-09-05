---
author: AI Cockpit maintainers
title: "WI-600 — v0.2.79 release と adopter acceptance"
description: "WI-599 の process-order 修正後の Release を公開し、immutable artifact を検証する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-600-release-v0-2-79
lastVerifiedBy: WI-600-release-v0-2-79
---

[English](WI-600-release-v0-2-79.md) · [简体中文](WI-600-release-v0-2-79.zh-CN.md)

# WI-600 — v0.2.79 release と adopter acceptance

## Objective

WI-599 の documentation-gate ordering 修正後、review 済みで同期された default
branch から v0.2.79 を公開します。source や workspace fallback なしに、公開 artifact
だけで新しい adopter を governance できることを確認します。

## Boundary

この Work Item は package version metadata と current release/versioning 文書だけを
変更します。Runtime source、object repository、global Agent/MCP configuration、
historical evidence bytes、reference-source implementation は範囲外です。公開後の
adopter/N-1 acceptance は download した immutable artifact だけを使います。

## Acceptance

1. Cargo metadata と lockfile は v0.2.79 になり、失敗した過去 tag は保持して再利用しません。
2. Release policy と hosted checks は annotated tag、5 target artifact、checksum、
   SBOM/provenance、Runtime identity を一つの review 済み commit に bind します。
3. Public adopter/N-1 harness は v0.2.79 artifact だけを使い、forbidden-root isolation と
   成功/失敗時の temporary-run cleanup を証明します。
4. 英語・簡体字中国語・日本語の current release、architecture、versioning 文書を一致させ、
   object repository は変更しません。
5. 公開後の失敗は公開済みの事実を保持して failure receipt を記録し、tag や historical
   evidence を書き換えません。

## Verification

Contract に列挙した locked workspace、documentation、parity、release policy、staged
acceptance、公開後 public-artifact check を実行します。hosted checks、v0.2.79 Release、
adopter/N-1 receipt 保存、正確な branch/worktree cleanup を確認してから lifecycle を完了します。
