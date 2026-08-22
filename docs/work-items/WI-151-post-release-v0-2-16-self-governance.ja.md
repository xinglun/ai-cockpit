---
author: AI Cockpit maintainers
title: "WI-151 — v0.2.16 post-release self-governance acceptance"
description: "immutable な公開 v0.2.16 binary だけを使い、install 後に AI Cockpit が本 repository を治理できることを確認する。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-151-post-release-v0-2-16-self-governance
workItemId: WI-151-post-release-v0-2-16-self-governance
---

# WI-151 — v0.2.16 post-release self-governance acceptance

WI-151 は post-release acceptance の境界です。公開された v0.2.16 aarch64 macOS archive を
download し、checksum と archive layout を検証した後、source や workspace fallback を使わずに
展開した binary を install しました。

install された binary identity は次のとおりです。

- version: `0.2.16`
- binary SHA-256: `0e9e9e85f3a96d22702cf95edab928bd2307c4636e53836bee46ca4e8cabf796`
- repositoryId: `sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b`

明示的な `--repo` で `inspect`、`status`、`doctor`、`agent doctor`、全 workspace verification が通過しました。
Human Outcome は English、Simplified Chinese、日本語で表示され、可視の `🟢` marker と structured Human
Decision を含みます。acceptance evidence は `.ai/evidence/WI-151-post-release-v0-2-16-self-governance.verification.json`、
decision は `.ai/decisions/WI-151-post-release-v0-2-16-self-governance.close.json` です。

Release workflow と公開 artifact が publication truth の authority であり、この Work Item は install 後の
adopter result を記録します。
