---
author: AI Cockpit maintainers
title: "WI-466 — v0.2.54 release と公開 adopter acceptance"
workItemId: WI-466-release-v0-2-54
description: "レビュー済み main から v0.2.54 Runtime を公開し、隔離した adopter flow で公開 binary を検証する。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: authorized
lastVerifiedBy: WI-466-release-v0-2-54
terminalArchive: .ai/work-items/archive/WI-466-release-v0-2-54.contract.json
terminalVerification: .ai/evidence/WI-466-release-v0-2-54.verification.json
terminalFinalization: .ai/decisions/WI-466-release-v0-2-54.finalize.json
terminalDecision: .ai/decisions/WI-466-release-v0-2-54.close.json
---

# WI-466 — v0.2.54 release と公開 adopter acceptance

## Intent

closed Work Item の documentation promotion 修正を含む patch を公開し、source や workspace fallback を使わずに公開 artifact が隔離 adopter repository を初期化・統治できることを証明する。

## Scope

- 三言語の workspace package identity と現在の installation guidance を `v0.2.54` に更新する。
- 同期済み `main` から annotated tag を push し、レビュー済み release workflow を実行して manifest、checksum、SBOM、provenance、tag evidence を保持する。
- 隔離した HOME、XDG_CONFIG_HOME、TMPDIR、CARGO_HOME、adopter repository で公開 artifact を install し、public adopter と N-1 acceptance を実行する。
- `first-adopter-smoke=not_ready`、Runtime/repository identity、evidence reuse、lifecycle receipt、cleanup proof を保持する。

## Out of scope

reference source checkout、object repository、global Agent/MCP configuration、Homebrew tap mutation、source fallback、Runtime architecture redesign、無関係な reference-parity batch。

## Acceptance criteria

1. Workspace package version と release documentation が予約済みの履歴を変更せず正確に `v0.2.54` へ進む。
2. レビュー済み release workflow が annotated tag、source commit、manifest、`SHA256SUMS`、SBOM、provenance、公開 artifact identity を bind する。
3. local strict、version、workflow、documentation、workspace tests が通り、source fallback を使わない。
4. merge 後、post-release adopter acceptance が公開 `v0.2.54` binary を download・checksum 検証し、Runtime identity と cleanup receipt を保持する。
5. close 後も Runtime repository は健全で `ready_on_base` である。

## Evidence and verification

terminal record は release tag と公開 artifact をレビュー済み source commit に bind する。Adopter evidence は `runtime.json`、repository/Work Item identity、lifecycle receipt、evidence reuse、isolation manifest、cleanup state を保持する。検証コマンドは次のとおり：

```text
cargo test --locked --workspace
```

公開 release と N-1 acceptance は post-release evidence であり、失敗しても Release truth を書き換えない。

## Boundary

`v0.2.54` は同一 schema の patch である。Runtime upgrade と repository attach は分離され、release publication は adopter repository を attach・変更しない。
