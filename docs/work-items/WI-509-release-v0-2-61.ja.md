---
author: AI Cockpit maintainers
title: "WI-509 — v0.2.61 release と public adopter acceptance"
description: "次の identity-bound Runtime release を公開し、reference parity 再開前に public artifact を検証します。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-509-release-v0-2-61
terminalArchive: .ai/work-items/archive/WI-509-release-v0-2-61.contract.json
terminalVerification: .ai/evidence/WI-509-release-v0-2-61.verification.json
terminalFinalization: .ai/decisions/WI-509-release-v0-2-61.finalize.json
terminalDecision: .ai/decisions/WI-509-release-v0-2-61.close.json
workItemId: WI-509-release-v0-2-61
---

# WI-509 — v0.2.61 release と public adopter acceptance

[English](WI-509-release-v0-2-61.md) · [简体中文](WI-509-release-v0-2-61.zh-CN.md)

## Intent

review 済み main から v0.2.61 を公開し、reference parity を再開する前に
隔離された adopter acceptance で immutable public artifact を検証します。

## Scope

- workspace package、lockfile、現在の三言語 release/versioning guidance を v0.2.61 にそろえ、過去の事実は保持します。
- 三言語の reference-parity ledger にこの release Work Item を登録します。
- synchronized main から annotated tag を作成する前に reviewed hosted PR を通過し、archive、checksum、SBOM、provenance、manifest を公開します。
- downloaded immutable artifact だけで public adopter と N-1 acceptance を実行し、isolation、evidence binding、not_ready scaffold、cleanup proof を含めます。
- public binary をこの repository に install し、inspect、status、doctor、Agent doctor、documentation-promotion health を確認します。

## Out of scope

local reference source、object/adopter repository、global Agent/MCP または Homebrew configuration、source/workspace binary fallback、無関係な Runtime redesign、generated governance record の手編集。

## Acceptance criteria

1. workspace package と lockfile が v0.2.61 を示し、現在の三言語 release guidance が過去の release history を書き換えず更新される。
2. synchronized main から annotated v0.2.61 tag を作成する前に、review 済み PR の hosted checks がすべて成功する。
3. public Release が identity-bound archive、SHA256 checksum、SBOM、provenance、release manifest を公開する。
4. public adopter と N-1 acceptance は downloaded immutable artifact だけを使い、isolation と temporary-root cleanup を証明し、first-adopter-smoke=not_ready を保持する。
5. public binary をこの repository に install した後も inspect、status、doctor、Agent doctor、post-close documentation check が healthy である。
6. publication 前に visible human Outcome、archive、finalization、close、exact branch/worktree cleanup がそろう。

## Verification

```text
cargo test --locked --workspace
```

Release publication と public acceptance は post-release evidence です。失敗した publication は immutable な failed history として保持し、成功として再ラベルまたは再利用しません。

## Boundary

Runtime binary は共有しますが、この repository の Protocol、Work Item、evidence、knowledge、adapter は repository ごとに分離します。Runtime の公開が他 repository を暗黙に attach または変更することはありません。
