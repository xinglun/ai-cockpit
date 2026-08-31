---
author: AI Cockpit maintainers
title: "WI-441 — local-reference entrypoint と Agent parity"
workItemId: WI-441-reference-entrypoint-parity
description: "local reference の Agent rule、reader route、capability boundary、Task Outcome entrypoint を再確認する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-441-reference-entrypoint-parity
terminalArchive: .ai/work-items/archive/WI-441-reference-entrypoint-parity.contract.json
terminalVerification: .ai/evidence/WI-441-reference-entrypoint-parity.verification.json
terminalFinalization: .ai/decisions/WI-441-reference-entrypoint-parity.finalize.json
terminalDecision: .ai/decisions/WI-441-reference-entrypoint-parity.close.json
---

# WI-441 — local-reference entrypoint と Agent parity

maintainer が管理する local reference checkout
`/Users/sei-rinn/dev/workspace_python/ai-cockpit-template` の 9 file を、commit
`fde3380f81fea5fd2e288f7a8849f737dc074060` に固定して一つずつ再確認します。public reference には接続しません。
これは Rust target の file-level semantic decision であり、source Agent file、Python/Make command、source JSON wire format を
copy する要求ではありません。

[English](WI-441-reference-entrypoint-parity.md) · [简体中文](WI-441-reference-entrypoint-parity.zh-CN.md)

## Scope と file-level decision

| Local reference path | Classification | Rust counterpart / boundary |
| --- | --- | --- |
| `AGENTS.md` | `implemented-different-by-design` | `AGENTS.md`、`.ai/README.md`、`docs/reference/agent-workflow.md`、typed lifecycle/adapter service が Contract-first、latest base、human pause、closure、cleanup を保持します。source `make ai-*` は target command ではありません。 |
| `GEMINI.md` | `implemented-different-by-design` | `.ai/README.md`、`crates/cockpit-agent`、install test、explicit Gemini adapter route が provider-facing guidance を提供します。provider 固有 file や global config は commit しません。 |
| `docs/README.md` | `implemented-different-by-design` | target の current/getting-started/operations/reference route が reader-first、goal-first の意図と Rust boundary を保持します。 |
| `docs/README.zh-CN.md` | `implemented-different-by-design` | 中国語 reader route と language link が同じ意図と Runtime/adopter boundary を保持します。 |
| `docs/README.ja.md` | `implemented-different-by-design` | 日本語 reader route と language link が同じ意図と Runtime/adopter boundary を保持します。 |
| `docs/capabilities.md` | `implemented-different-by-design` | target capability page は Repository Governance Layer と external non-claim を保持し、Rust CLI、MCP、scaffold、profile、knowledge、Outcome、isolation route を説明します。 |
| `docs/capabilities.zh-CN.md` | `implemented-different-by-design` | 中国語 capability route は source boundary と repository-local Runtime/adopter inheritance を保持し、source status を copy しません。 |
| `docs/capabilities.ja.md` | `implemented-different-by-design` | 日本語 capability route は source boundary と repository-local Runtime/adopter inheritance を保持し、source status を copy しません。 |
| `docs/features/task-outcome-report.md` | `implemented-different-by-design` | `OutcomeV2`、CLI/MCP human handoff、immutable evidence が report/status/PR separation を保持します。source prose と Make command は wire requirement ではありません。 |

## Boundary decision

Current reference は specification corpus であり、private local source です。Rust repository に source file がないことは、portable な責任が
shared Runtime、repository-local `.ai/` route、または explicit provider adapter で提供されていれば omission ではありません。source 固有の
command と generated record は reference-only のままです。adopter repository も同じ境界を継承します。一つの Runtime、明示的な repository context、
repository-local evidence、owner が明示的に install した場合だけ provider-owned Agent configuration を使います。

## Verification boundary

inventory は `sourceChangedSincePrevious`、`previousBatch`、`previousClassification` を audit history として保持し、本 batch の 9 current record を解決します。
local source policy は clean checkout、pinned commit、`network_access = false` を要求します。hosted CI は committed offline corpus だけを使い、documentation、parity、
governance-integrity、status-consistency、locked workspace test が verification evidence です。
