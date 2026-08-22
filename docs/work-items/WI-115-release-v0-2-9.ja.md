---
author: AI Cockpit maintainers
title: "WI-115 — v0.2.9 Release と capability surface parity"
description: "v0.2.9 を公開し、reference source の command、MCP、Release documentation parity の不足を解消する。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-115-release-v0-2-9
capabilityClaims:
  - release_distribution
  - reference_parity
  - cli_commands
---

# WI-115 — v0.2.9 Release と capability surface parity

## Goal

review 済み default branch から次の immutable Release を公開し、reference source
の Agent rule、CLI/MCP capability inventory、Release example を future Work Item と
adopter に対して正確で継承可能にする。

## Scope

- v0.2.9 の三言語 version/release distribution documentation、current baseline、N-1 example;
- 三言語 MCP/CLI command inventory（`delegated_evidence_list`、`capability show`、`diagnose` を含む）;
- feature と reference-parity における Runtime 検証済み `humanHandoff` boundary;
- inventory と Release target example の drift を防ぐ documentation acceptance check;
- downloaded public artifact だけを使う immutable v0.2.9 Release、adopter acceptance、
  v0.2.8→v0.2.9 N-1 acceptance。

## Out of scope

Runtime behavior、Protocol schema、global Agent/MCP configuration、external Homebrew tap mutation、
historical Release/evidence の書き換え、second-technology-stack adopter。

## Findings addressed

reference comparison により、one Work Item/branch/worktree/PR、explicit repository binding、
fail-closed preflight/Outcome、current WI 内の defect repair、immutable Release acceptance、
global Agent/MCP write 禁止は既に継承済みと確認しました。残りは documentation drift です。
MCP tool 一つと CLI entry 二つの不足、Release target example の曖昧さ、MCP の人間向け projection
を Agent layer が生成するとした古い説明を修正します。

## Acceptance

1. 三言語 capability page が `tools/list` の 12 tools を列挙し、CLI reference が
   `capability show` と `diagnose` を列挙する。
2. Release page が v0.2.9 を current とし、complete adopter baseline example に
   `x86_64-unknown-linux-gnu` を使う。他 target は追加 coverage と明記する。
3. feature/parity page が Runtime が OutcomeV2 を検証し `humanHandoff` を生成すること、
   Agent/conversation layer は選択・表示だけを行い governance authority にはしないことを明記する。
4. documentation、version consistency、Release policy、Rust quality、conformance、adopter harness が pass する。
5. 公開 v0.2.9 artifact が adopter/N-1 acceptance に合格し、repository/runtime isolation、
   cleanup、evidence reuse、`first-adopter-smoke = not_ready` を証明する。
6. installed Runtime lifecycle を完了し、🟢🟡🔴、unknown、evidence、decision、next action を含む
   人間向け Outcome を表示する。

## Inheritance boundary

Future Work Item は `AGENTS.md`、`.ai/README.md`、`docs/reference/agent-workflow.*` を継承します。
これらが repository-local の operation authority であり、この record は Release evidence です。

## Release truth

既存の v0.2.8 Release と pre-fix/failure receipt は immutable のまま保持します。post-release adopter
failure は failed evidence として記録し、公開済み Release truth を書き換えません。

