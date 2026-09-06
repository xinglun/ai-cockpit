---
title: "WI-603 — v0.2.80 release と adopter acceptance"
description: "次の Rust Runtime patch を公開し、immutable artifact と adopter 境界を検証する。"
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-603-release-v0-2-80
lastVerifiedBy: WI-603-release-v0-2-80
terminalArchive: .ai/work-items/archive/WI-603-release-v0-2-80.contract.json
terminalVerification: .ai/evidence/WI-603-release-v0-2-80.verification.json
terminalFinalization: .ai/decisions/WI-603-release-v0-2-80.finalize.json
terminalDecision: .ai/decisions/WI-603-release-v0-2-80.close.json
---

[English](WI-603-release-v0-2-80.md) · [简体中文](WI-603-release-v0-2-80.zh-CN.md)

# WI-603 — v0.2.80 release と adopter acceptance

## 目的

レビュー済み Rust Runtime を `v0.2.80` として公開し、immutable な公開
Release artifact だけで install と upgrade の境界を検証する。これは参照
ソースの逐次 parity と文書収束後の release boundary であり、source scaffold
の移植ではない。

## 境界

対象は package version、release/distribution、architecture/versioning 文書、
parity 登録、および staged/public acceptance evidence である。参照プロジェクト
の scaffold、Python/Make 実装、JSON wire format はコピーしない。object/adopter
repository は外部の read-only acceptance target とし、global Agent/MCP 設定と
歴史的 governance bytes は対象外とする。

## Acceptance

1. Workspace package と `Cargo.lock` が `0.2.80` に解決される。
2. annotated tag、5 target archive、SBOM/provenance、Formula、checksum、manifest、
   Runtime identity が Release CI で相互に bind される。
3. staged/public adopter acceptance は download 済み artifact のみを使い、repository
   isolation と一時 run の cleanup を証明する。失敗時も公開済み Release truth を書き換えない。
4. 英語・中国語・日本語の release、architecture、versioning、parity 文書が `v0.2.80`
   と N-1 `v0.2.79` の境界で一致する。
5. hosted checks 後に公開 Release を検証し、参照 checkout と object repository を変更しない。

## 検証

`cargo test --locked --workspace`、documentation/metadata、release policy/version
consistency、および immutable staged/public adopter harness を実行する。Release
evidence には download した Runtime の version と SHA-256 を記録する。terminal
Outcome は status、unknowns、evidence、decision、next action を含む、人向けの独立した
可視 handoff とする。

