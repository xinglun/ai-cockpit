---
author: AI Cockpit maintainers
title: "WI-301 — v0.2.33 public Release adopter acceptance"
workItemId: WI-301-release-adopter-acceptance
description: "隔離した新規 adopter で immutable な v0.2.33 binary を検証し、公開 N-1 upgrade を確認します。"
audience:
  - maintainer
  - reviewer
  - adopter
status: implemented
lastVerifiedBy: WI-301-release-adopter-acceptance
terminalArchive: .ai/work-items/archive/WI-301-release-adopter-acceptance.contract.json
terminalVerification: .ai/evidence/WI-301-release-adopter-acceptance.verification.json
terminalFinalization: .ai/decisions/WI-301-release-adopter-acceptance.finalize.json
terminalDecision: .ai/decisions/WI-301-release-adopter-acceptance.close.json
authority: canonical
---

# WI-301 — v0.2.33 public Release adopter acceptance

## Intent

公開済みで immutable な v0.2.33 Release binary が新しい repository をゼロから
govern でき、公開 v0.2.31 binary で作成した repository を履歴 evidence を失わず
upgrade できることを確認します。

## Scope

この acceptance は `aarch64-apple-darwin` で download した public Release artifact
だけを使います。archive と executable の SHA-256、repository/Runtime identity、
attach/profile/Agent doctor 出力、`first-adopter-smoke` の `not_ready` Contract
skeleton、evidence reuse、完全な Work Item lifecycle、N-1 upgrade の履歴保持、
isolation manifest、temporary root cleanup を記録します。

receipt は次に保持します。

- `.ai/evidence/external/v0.2.33/adopter-aarch64-apple-darwin/`
- `.ai/evidence/external/v0.2.33/upgrade-v0.2.31-to-v0.2.33/`

## Evidence boundary

`runtime.json` は tag `v0.2.33`、archive digest
`sha256:c8019db3d8509d62418afed114b986689df7b0ef570ff7199a4b845c7d932ca4`、展開した
binary digest
`sha256:eceed75ef74079e7ede420b42f8223fc76be82ec0211ddc6b8fdf7cb3c3b9de4` を binding
します。upgrade receipt は public N-1 tag `v0.2.31` と v0.2.33 を binding し、旧
evidence bytes を byte-for-byte で保持します。`acceptance.json` は
`releasePublished: true`、`adopterAcceptance: passed`、`cleanupState: passed` を
報告します。post-release の失敗は failed evidence として残り、Release truth を
rewrite できません。

HOME と XDG_CONFIG_HOME は write-forbidden root、TMPDIR と CARGO_HOME は明示的に
隔離した Runtime-write root です。cleanup receipt は、failure-safe path を含め、
検証済み temporary `run_root` がすべて削除されたことを示します。

この harness は post-release evidence です。second technology-stack adopter は別の
Work Item で扱い、本記録では主張しません。
