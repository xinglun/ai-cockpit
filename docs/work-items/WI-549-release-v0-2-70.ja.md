---
author: AI Cockpit maintainers
title: "WI-549 — v0.2.70 release と公開 artifact acceptance"
description: "次の immutable Runtime baseline を公開し、download-only の公開 artifact で受入れを行う。"
audience: [maintainer, reviewer, adopter]
status: in-progress
authority: canonical
workItemId: WI-549-release-v0-2-70
lastVerifiedBy: WI-549-release-v0-2-70
---

[English](WI-549-release-v0-2-70.md) · [简体中文](WI-549-release-v0-2-70.zh-CN.md)

# WI-549 — v0.2.70 release と公開 artifact acceptance

## 目的

レビュー済みの default branch から v0.2.70 を次の immutable Runtime baseline
として公開します。失敗した v0.2.68 tag は history として保持し、再利用しません。

## 範囲と境界

- workspace package identity と lockfile を v0.2.70 に更新します。
- English、簡体字中国語、日本語の release/versioning 文書を tag、checksum、
  SBOM、provenance、attestation、adopter acceptance の手順を含めて一致させます。
- 公開を、close と documentation promotion が済んだ WI-548 reference-parity
  batch に束縛します。
- 対象 repository、global Agent/MCP 設定、source template の copy、Runtime
  behavior の変更はこの Work Item の範囲外です。

## Release acceptance

1. package metadata、lockfile、release 文書が v0.2.70 を示し、v0.2.68 は
   immutable な失敗履歴として残ります。
2. Release CI が release manifest、SHA256SUMS、SBOM、provenance、attestation、
   tag/source/Runtime identity binding 付きの公開 artifact を生成します。
3. post-release adopter と N-1 harness は隔離 root で download-only の公開
   artifact だけを使い、forbidden root isolation と cleanup を証明します。
4. 同じ公開 binary が明示的な repository context で本 repository を inspect、
   status、doctor し、運用できます。

## 検証境界

Contract の acceptance prose は元の言語を保持します。localized page は表示ラベル
だけを変え、governance fact を翻訳・変更しません。失敗した公開は失敗として記録し、
tag を再利用しません。対象 repository の acceptance は外部 read-only step であり、
対象チームの receipt が到着するまで成功とは主張しません。
