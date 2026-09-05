---
author: AI Cockpit maintainers
title: "WI-591 — v0.2.77 release と object-adopter recovery handoff"
description: "predecessor close 再検証修正を含む Runtime を公開し、public artifact を検証します。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-591-release-v0-2-77
lastVerifiedBy: WI-591-release-v0-2-77
---

[English](WI-591-release-v0-2-77.md) · [简体中文](WI-591-release-v0-2-77.zh-CN.md)

# WI-591 — v0.2.77 release と object-adopter recovery handoff

## 目的

review 済みで同期された default branch から v0.2.77 を公開します。この Release
には WI-589 の Contract amendment predecessor-close revalidation 修正を含め、
immutable な Release evidence を保持し、object repository 向けに read-only の
受入れ handoff を用意します。

## 境界

この Work Item は package version metadata と Release 文書だけを変更します。
Runtime 実装、object repository、global Agent/MCP 設定、historical evidence bytes、
reference-source 実装は範囲外です。public adopter と N-1 acceptance は公開後の
evidence であり、source checkout や workspace build ではなく download 済みの
immutable artifact だけを使用します。

## 受入れ

1. Cargo metadata、lockfile、英中日の release/versioning guide が v0.2.77 を示し、
   v0.2.76 を直前の public baseline として保持します。
2. Release policy check が annotated tag、5 target artifact、checksum、SBOM/provenance、
   Runtime identity を同一 source commit に bind することを証明します。
3. 公開後の adopter/N-1 harness は v0.2.77 の public artifact だけを使い、forbidden
   root isolation と temporary-run cleanup を証明します。
4. object repository は変更せず、公開後に正確な compatibility/recovery/revalidation
   command をチームへ渡します。
5. Release または adopter が失敗しても公開済みの事実を保持し、failure receipt を
   記録します。失敗 tag や historical evidence は書き換えません。

## 検証

Contract に列挙した Release policy、documentation、parity、locked workspace check を
実行します。公開後は v0.2.77 に対して public adopter と N-1 acceptance harness を
実行し、immutable receipt を保存します。
