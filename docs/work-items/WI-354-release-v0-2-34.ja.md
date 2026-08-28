---
author: AI Cockpit maintainers
title: "WI-354 — v0.2.34 release preparation"
workItemId: WI-354-release-v0-2-34
description: "lifecycle cleanup guard 後の v0.2.34 release を準備し、公開 artifact 受入れを post-release successor に引き継ぐ。"
audience: [maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-354-release-v0-2-34
capabilityClaims: [release_distribution]
---

# WI-354 — v0.2.34 release preparation

[English](WI-354-release-v0-2-34.md) · [简体中文](WI-354-release-v0-2-34.zh-CN.md)

## Intent と境界

WI-352 で lifecycle cleanup guard が閉じた後の reviewed default branch から
v0.2.34 を準備し、workspace version と current installation documentation
を揃え、reviewed hosted release route だけで公開します。

この Work Item は過去の Release truth を書き換えず、tag 公開前に public
artifact の install 成功を主張しません。公開後の successor が immutable な
public archive を download・install し、current repository と adopter の境界を
検証します。

## Scope

- `Cargo.toml`、`Cargo.lock` と三言語の release、distribution architecture、
  versioning の current route を v0.2.34 に揃える。
- v0.2.30 と v0.2.32 の失敗した公開履歴を保持する。
- tag 前に documentation、version consistency、governance integrity、
  release policy、workspace の全 gate を実行する。
- `.github/workflows/release.yml` で reviewed tag だけを公開し、manifest、
  checksum、SBOM、provenance、archive smoke、staged adopter evidence を bind する。
- public binary install と current repository acceptance を post-release successor に渡す。

## Out of scope

WI-351/WI-353 recovery、新しい Runtime governance、外部 Homebrew tap、global
Agent/MCP 設定、second technology adopter、post-release receipt の内容は対象外です。

## Acceptance と verification

- workspace 全 package と `Cargo.lock` が 0.2.34、tag が `v0.2.34` になる。
- 三言語の current release、distribution architecture、versioning が v0.2.34
  を示し、過去の失敗事実を保持する。
- 公開前の source route と hosted release gate がすべて pass する。
- tag workflow が manifest、`SHA256SUMS`、五つの target archive、SBOM、
  provenance、staged adopter gate を同一 commit に bind する。
- この Work Item は post-release install 成功を記録しない。その結論は immutable
  public-artifact successor に属する。

宣言した検証は `cargo test --locked --workspace`、documentation/release
consistency、release policy test、hosted quality、Windows、behavioral-oracle、
archive、SBOM、staged-adopter jobs です。
