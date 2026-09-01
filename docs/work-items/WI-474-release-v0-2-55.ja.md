---
author: AI Cockpit maintainers
title: "WI-474 — v0.2.55 release と公開 adopter acceptance"
description: "レビュー済み Runtime patch を公開し、adopter repository を変更せず immutable な公開 binary を検証する。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: authorized
lastVerifiedBy: WI-474-release-v0-2-55
terminalArchive: .ai/work-items/archive/WI-474-release-v0-2-55.contract.json
terminalVerification: .ai/evidence/WI-474-release-v0-2-55.verification.json
terminalFinalization: .ai/decisions/WI-474-release-v0-2-55.finalize.json
terminalDecision: .ai/decisions/WI-474-release-v0-2-55.close.json
workItemId: WI-474-release-v0-2-55
---

# WI-474 — v0.2.55 release と公開 adopter acceptance

## Intent

次のレビュー済み Runtime patch を公開し、immutable な公開 binary を install
して隔離した adopter を治理できることを証明する。この release は mainline の
reference comparison を継続するもので、reference source や adopter repository
は変更しない。

## Scope

- workspace package identity と現在の三言語 release/versioning guidance を
  `v0.2.55` に進め、過去の release facts を保持する。
- archive 前にこの Work Item を三言語の reference-parity ledger に登録する。
- レビュー済み PR を merge し、annotated tag を公開し、manifest、checksum、
  SBOM、provenance、artifact identity evidence を保持する。
- 隔離 root で download した公開 Release artifact だけを使って public adopter
  と N-1 acceptance を実行し、evidence reuse と temporary-root cleanup を証明する。
- この repository に公開 binary を install/upgrade し、repository、Runtime、
  Agent、readiness state を検証する。

## Out of scope

ローカル reference source、`/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator`、
他の adopter repository、global Agent/MCP configuration、Homebrew tap mutation、
source fallback、無関係な reference-parity または Runtime architecture の変更は対象外である。

## Acceptance criteria

1. workspace package、lockfile、必須の三言語 release/versioning document が
   `v0.2.55` を示し、歴史を rewrite しない。
2. PR は merge 前に hosted checks を通過し、annotated `v0.2.55` tag は同期済みの
   レビュー済み main commit を正確に指す。
3. 公開 Release は予定された archive、checksum、SBOM、provenance metadata、
   identity-bound release manifest を提供する。
4. public adopter と N-1 acceptance は immutable な公開 artifact だけを使い、
   `first-adopter-smoke=not_ready`、repository/Runtime identity、forbidden-root isolation、
   成功/失敗両方の temporary-root cleanup を証明する。
5. 公開 binary をこの repository に install した後、`inspect`/`status`/`doctor`/
   `agent doctor` が attached、healthy、isolated、`ready_on_base` を確認する。
6. 可視の human Outcome、archive/finalization/close、documentation promotion、
   正確な branch/worktree cleanup を完了する。

## Verification

Source verification command:

```text
cargo test --locked --workspace
```

Release publication と public adopter acceptance は post-release evidence であり、
失敗しても Release truth を書き換えない。

## Boundary

Runtime upgrade は shared executable の交換だけを行い、Repository Protocol state は
repository ごとに保持する。Publication は adopter repository を attach/変更しない。
