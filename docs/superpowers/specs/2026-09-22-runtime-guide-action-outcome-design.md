# Runtime Action Explanation, Task Guides, and Outcome Handoff Design

## Goal

让 Runtime 负责当前状态、准入和证据有效性，让任务指南负责适用场景与操作引导，让 Outcome 负责完整的人与 AI 交接，同时降低普通 Work Item 的默认阅读、重复判断和收尾成本。

本设计只改变当前仓库的入口、增量 Runtime 投影、机械 Reference、文档投影边界和 Outcome 验收。它不发布、不改变已关闭 Work Item、不重写历史 Contract/receipt/archive，也不引入独立规划器、通用插件框架或新的文档治理平台。

## Fixed execution identity and baseline

实施分支从 origin/main@74ce5abed00385c92b026d62aa8893d92f78b570 创建。整个 Work Item 固定使用：

| Fact | Value |
| --- | --- |
| Runtime executable | /Users/sei-rinn/.local/bin/ai-cockpit |
| Runtime version | 0.2.105 |
| Binary SHA-256 | 43a8ed731ac3187868e8d0c021be2a1a1b72986885d13bf3373f2ed5a0296b04 |
| Runtime identity | sha256:43a8ed731ac3187868e8d0c021be2a1a1b72986885d13bf3373f2ed5a0296b04 |
| Repository identity | sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b |
| Repository source identity | 74ce5abed00385c92b026d62aa8893d92f78b570 |
| Repository state | main and origin/main aligned; clean before Work Item start |

The repository tag v0.2.106 is historical release context. It is not a permission to switch Runtime binaries, and the Work Item must not mix the fixed 0.2.105 executable with another installed Runtime.

The current default static read set is:

| Input | Bytes |
| --- | ---: |
| AGENTS.md | 9,362 |
| .ai/README.md | 8,157 |
| .ai/glossary.md | 3,881 |
| .ai/agent-interface.json | 681 |
| Total | 22,081 |

This is the pre-change byte baseline. Token counts are intentionally not used. The pilot measurement records command invocations, read-only query invocations, verification process starts, phase durations, exit status, and whether each decision came from Runtime output or Agent inference. The measurement is recorded here and in the final Outcome; it is not a new permanent registry.

After the A entry split and final route wording, the ordinary Work Item
default read set is 13,172 bytes: the public entry files, the task-guide index,
and the required `ordinary-work-item` guide. This is a 40.3% reduction (8,909
bytes) without relying on whitespace compression. The glossary is now on
demand rather than part of the ordinary default set. The ordinary route still
uses the existing lifecycle boundaries; the new action gate is a fresh
in-process query, not another verification command. The fixed source build's
warm status benchmark currently reports 12 samples, median 121 ms, and P95
144 ms. This is a current-worktree measurement, not a release or
hosted-runtime claim.

The pilot cost record is intentionally finite and evidence-based:

| Measurement | Result | Source or limitation |
| --- | --- | --- |
| Ordinary default static read set | 22,081 -> 13,172 bytes; 8,909 bytes / 40.3% lower | `tests/docs/governance_cost_baseline_test.py`; includes the task-guide index and required ordinary guide, with glossary on demand |
| Current Contract verification declarations | 8 | Runtime-owned active Contract; a historical command-count baseline was not recorded, so no stronger no-increase claim is made |
| Read-only status query | 2 successive queries; 0 verification processes; files unchanged; status digest and safe actions stable | local comparison wrapper against the fixed Runtime; query is not an execution receipt |
| Plan-only admission check | 1 planned node; 0 verification processes | fixed Runtime plan-only result |
| Final formal verification | 1 planned/executed node; 1 process; exit 0; 239,654 ms execution / 239,952 ms receipt elapsed | [verification receipt](../../../.ai/evidence/WI-1006-runtime-guide-action-outcome.verification.json) |
| Preserved negative verification evidence | malformed argv: exit 101 in 89 ms; cold attempt: timed out at 300,279 ms; precondition rejections: 0 processes | Runtime-generated attempt receipts; these remain evidence, not success |
| Documentation phase | 7 cheap checks, 4.3 s wall time; promotion fixture 3.999 s | measured in the current worktree; checks were independent and run concurrently |
| Action/Outcome focused phase | repository action/projection 22.365 s; CLI action/Outcome 15.684 s; MCP read-only 0.678 s; Agent delivery 0.336 s | targeted Rust test runs; cargo build/cache state affects elapsed time |

The exact historical count of required commands and full verification runs was
not captured before implementation. The current Contract retains eight
declared checks and the final formal receipt proves one bounded full-workspace
run; this is reported as a limitation rather than inferred as a regression
comparison. Runtime facts (state, blockers, safe actions, evidence freshness,
and admission identity) come from the Runtime. Guide selection and human
Outcome wording come from the Agent/adapter. No manual task ledger or
independent task-progress API is part of this record.

The fixed 0.2.105 Runtime also exposed a capability mismatch in the existing
Contract declaration: `verify --plan-only` rejects a declaration containing the
literal `<repo>` placeholder as a runnable argv command. The final formal run
therefore used the real explicit `cargo` command/arguments at the Runtime
boundary and preserved the declaration mismatch as a limitation; no generated
Contract or hand-written ledger was altered to conceal it.

The pre-change command/reference baseline also has a known independent discrepancy: `python3 scripts/generate_interface_references.py --repo /Users/sei-rinn/.codex/worktrees/wi-runtime-guide-action-outcome/ai-cockpit --check` reports drift for `docs/reference/commands.md`, `docs/reference/commands.zh-CN.md`, and `docs/reference/commands.ja.md`. The implementation must preserve this as a separate baseline fact, inspect the generated region, and repair only mechanically owned content from the fixed authoritative definition.

## Responsibility boundaries

| Layer | Owns | Must not own |
| --- | --- | --- |
| Runtime | current state, preconditions, action admission, evidence validity, blocker classification, observation identity | user authorization inferred from prose or recommendations |
| Contract/decision records | intent, scope, acceptance, authority, and amendments | execution evidence |
| AGENTS.md | cross-task boundaries, Runtime entrypoint, guide index, Outcome delivery boundary | a second complete lifecycle |
| .ai/README.md | repository attachment, configuration, record locations, concise entry route | another operating rulebook |
| Task guides | applicability, required inputs, ordered operations, diagnostics, stop/continue conditions, deep references | state machine, authorization, or action allow-list |
| Reference | protocol semantics, compatibility, design reasons, generated command facts | deciding whether this Work Item may continue |
| User documentation | how to use and understand the system and make decisions | requiring knowledge of internal governance implementation |
| Outcome/adapter | complete human-readable result and host delivery boundary | converting generated or returned content into false display confirmation |

## Architecture

### A. Reader-first entry and task guides

Create agents/skills/README.md as the stable route and four Markdown guides:

- ordinary-work-item.md for ordinary implementation, verification, archive, close, projection, and cleanup;
- verification-failure-recovery.md only for failed, timed-out, stale, malformed, unsupported, or contradictory evidence;
- provider-resource-finalization.md only when a Contract binds a provider PR/branch/worktree or another external resource;
- release-upgrade-acceptance.md only when the Contract explicitly declares release or upgrade acceptance.

Each guide has the same six sections: applicability, authoritative inputs, ordered operations and purpose, success conditions, evidence to preserve on failure, and continue/stop/human-decision rules. Guides link to Reference pages instead of copying the state machine or complete command inventory.

AGENTS.md will keep the public boundary, explicit Runtime query route, guide index, and complete Outcome delivery rule. .ai/README.md will keep repository attachment/configuration and record locations. The glossary will retain only terms required to understand that route and link to deeper Reference. Historical docs/work-items, archives, receipts, and multilingual historical projections are not migrated.

### B. Shared Runtime action explanation

Extend the existing WorkItemStatusSnapshot additively. Keep safeActions, blockers, evidence freshness, digests, diagnostics, and unknowns unchanged for existing consumers. Add an optional structured explanation containing:

- recommendedAction and stable recommendationReason;
- guideId;
- admissionState distinguishing allowed, blocked, human-decision-required, and unknown;
- typed issue categories for missing, unknown, malformed, unsupported, and contradictory inputs;
- explicit human decision request and missing inputs where applicable;
- an admission digest bound to the current Contract, repository snapshot, Runtime identity, and existing status facts.

The projection is derived from the same internal admission evaluation used by execution entrypoints. Querying is read-only and never starts verification, repairs state, records recovery, or grants permission. Execution recomputes the admission against a fresh snapshot and Contract; it never trusts a prior query or recommendation. Guide lookup is explanatory only: a missing guide or failed localization cannot change safeActions or admission.

The aggregate `admissionState` is a status explanation, while action-level
authority is the fresh Runtime `safeActions` set. This distinction is needed
for an archived item whose aggregate state is blocked but whose explicit
`close_after_review` or `close_after_cleanup` action is the admitted way to
resolve the current boundary. A recommendation remains only a projection;
the execution entrypoint still requires the requested action to be present in
the fresh set.

CLI and MCP expose the same language-neutral structured projection. Localized prose remains a presentation layer. Existing string safeActions remain available as an additive compatibility field.

### C. Mechanical projection and Outcome handoff

Close-time documentation projection receives an explicit Work Item identity and an explicit derived-file allow-list. It may update only derived Work Item documents/parity projections selected by the current policy. It never edits Contract, receipt, archive, close, evidence, or historical recovery bytes.

The projection path records pending synchronization separately when a write fails. A closed Work Item remains closed. Repeating the same projection with the same input identity produces no diff and cannot create a successor or a new projection Work Item. Tests use a fixture with two consecutive sync calls and compare file manifests, bytes, and successor records.

Runtime Outcome remains the single source for status, evidence, unknowns, human decision, and next action. The adapter forwards the complete prepared body and ordered assistant-message events. Return-only hosts retain hostConfirmation=unknown and displayConfirmation=unknown; generated, returned, accepted, and displayed remain distinct facts. Completion, external cleanup, projection synchronization, and host visibility are reported as separate domains.

## Authority and source-of-truth rules

- Command names, arguments, enums, and wire fields come from the existing Clap/protocol/schema/capability definitions. The generator only projects marked mechanical regions.
- Human explanations, design reasons, guide diagnostics, and impact language remain manually reviewed.
- No new describe command is introduced.
- A recommendation never authorizes an action.
- Unknown, missing, malformed, unsupported, and contradictory inputs remain distinct in machine output and human Outcome.
- Historical evidence is read-only. Compatibility failures are reported as compatibility failures, not as fabrication or contradiction.

## Acceptance scenarios

1. Ordinary no-resource Work Item: the ordinary guide is sufficient; release/recovery guides are not required; implementation, verification, closure, projection, and Outcome domains remain separate.
2. Verification failure or timeout: failure evidence remains preserved, recovery guide is selected, completion is not claimed, and the resume decision is explicit.
3. Query then state change: execution recomputes admission and rejects stale assumptions with the changed digest/evidence.
4. Missing authorization: the query explains the required human decision; recommendation does not unlock execution.
5. Provider cleanup pending: implementation and verification may be complete while resource cleanup remains pending.
6. Unsupported historical receipt: output identifies unsupported compatibility without relabeling it as fraud or contradiction.
7. Repeated projection: the second sync has no changes and creates no recursive successor.
8. CLI/MCP same state: core fields, blockers, safe actions, recommendation, guide, and admission agree.
9. Outcome with no host display confirmation: full ordered content is delivered or returned, but display is not claimed.

## Cost and verification

The success budget is a governance-cost budget, not an unmeasured speed claim:

- ordinary default static read bytes decrease by at least 40% from 22,081;
- ordinary success path required command count and full verification count do not increase;
- status/query paths start zero verification processes;
- fixed-environment query latency is reported with sample count, median, and P95;
- repeated projection is byte-idempotent;
- focused format, link, generated-reference, and targeted regression checks run before formal Contract verification;
- formal verification uses only the fixed Runtime identity and refreshed Contract/snapshot bindings.

No release, provider mutation, push, CI dispatch, PR creation/merge, tag mutation, global adapter configuration, or historical record rewrite is part of this Work Item.
