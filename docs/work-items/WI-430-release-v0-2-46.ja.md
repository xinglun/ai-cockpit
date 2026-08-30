---
author: AI Cockpit maintainers
title: WI-430 — v0.2.46 release
description: WI-429 の historical recovery 修正を immutable な Runtime Release として公開する。
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
workItemId: WI-430-release-v0-2-46
lastVerifiedBy: WI-430-release-v0-2-46
---

# WI-430 — v0.2.46 release

[English](WI-430-release-v0-2-46.md) · [简体中文](WI-430-release-v0-2-46.zh-CN.md)

## Intent

レビュー済みの WI-429 recovery-history 修正を v0.2.46 として公開し、adopter が immutable な public artifact から修正済み Runtime を install できるようにします。

## Boundary

この Work Item は一つの patch release と release 文書の同期だけを扱います。governance semantics、reference/V1 runtime のコピー、global Agent/MCP 設定、adopter application source は変更しません。

## Acceptance

- Cargo metadata と lockfile を v0.2.45 から v0.2.46 へ一つの patch として進め、既存 tag/Release を再利用しない。
- reviewed commit、五つの target archive、SBOM、manifest、Formula、checksum、provenance、Release identity を正確に binding する。
- release、installation、versioning、parity 文書を English、簡体中文、日本語で同期し、v0.2.45 は historical として保持する。
- post-release acceptance は immutable な public v0.2.46 artifact だけを使い、source/workspace/local binary fallback を拒否する。
- merge、finalize、close、同期、exact cleanup 後に `main` が `ready_on_base` になる。

## Verification boundary

release 前は Contract が定めた release gate を使います。public adopter acceptance は別 Work Item とし、Runtime identity、artifact digest、isolation manifest、cleanup proof を保持します。
