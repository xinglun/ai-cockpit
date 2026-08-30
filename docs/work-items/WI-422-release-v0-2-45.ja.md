---
author: AI Cockpit maintainers
title: WI-422 — v0.2.45 release
description: mixed-monorepo reference batch 後の reviewed Runtime を公開する。
workItemId: WI-422-release-v0-2-45
audience: [adopter, maintainer, reviewer]
status: release-preparation
authority: human-authorized
lastVerifiedBy: WI-422-release-v0-2-45
---

# WI-422 — v0.2.45 release

[English](WI-422-release-v0-2-45.md) · [简体中文](WI-422-release-v0-2-45.zh-CN.md)

## Intent

mixed-monorepo reference の逐 file 比較 batch 完了後に reviewed な `main` を v0.2.45 として公開し、
Release identity、installation guidance、三言語 parity record を同期する。

## Scope と boundary

この Work Item は一つの patch release と既存 strict release route の検証だけを扱う。
Runtime governance semantics、reference/V1 Runtime や installer のコピー、global Agent/MCP
configuration、adopter application source は変更しない。public-artifact adopter acceptance は
release 後の別 Work Item で、immutable な v0.2.45 artifact だけを使う。

## Acceptance

- Cargo metadata と lockfile を v0.2.44 から v0.2.45 へ一つだけ patch 更新し、既存の tag/Release を再利用しない。
- reviewed workflow が exact reviewed commit、target archive、SBOM、manifest、Formula、checksum、
  provenance、immutable tag/Release identity を bind する。
- release、installation、versioning、parity guidance を English、Simplified Chinese、日本語で同期し、
  新しい public adopter baseline の受入れまでは v0.2.44 を historical evidence として保持する。
- 後続の隔離 Work Item は immutable な public v0.2.45 artifact だけで post-release acceptance を行い、
  source/workspace/local binary fallback を使わない。
- reviewed merge、finalization、close、default branch 同期、exact cleanup の後に `main` が `ready_on_base` になる。

## Verification boundary

release 前は Contract が定めた strict source/release gate を使う。staged candidate や source build は
public adopter evidence として扱わない。post-release receipt は Runtime identity、artifact digest、
isolation manifest、cleanup proof を保持し、Release truth を書き換えない。
