---
author: AI Cockpit maintainers
title: "Reference File Comparison"
description: "The pinned, staged method for comparing the reference source file by file."
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - reference_parity
---

# Reference file comparison

This page explains how the Rust project compares itself with the public
reference source one file at a time. The reference is a specification and
behavior corpus; it is not a directory to copy into the Rust Runtime.

## Pinned baseline

- Reference: [`spirex-ds-dev/ai-cockpit-template`](https://github.com/spirex-ds-dev/ai-cockpit-template) at `e5acb677da6621004d96f0ef353c58fe8d3acfbf`.
- Rust comparison baseline: [`xinglun/ai-cockpit`](https://github.com/xinglun/ai-cockpit) `origin/main` at `a533d49dfa848d95742833f8cd1b5f7e1bb897d5`.
- Runtime used for the comparison work: `ai-cockpit 0.2.33`, binary SHA256 `eceed75ef74079e7ede420b42f8223fc76be82ec0211ddc6b8fdf7cb3c3b9de4`.

This page reports only the current pinned comparison baseline. Historical
delivery details are retained in Work Item archive evidence, not in this
reader-facing route.

The machine-readable ledger is
[`reference_file_inventory.json`](../../tests/conformance/reference_file_inventory.json).
Its regression check requires one classification for every tracked reference
path and rejects an unclassified first-batch path. Target checkout metadata is
derived from the pinned commit, not from dirty or untracked working-tree files.

## Classification rules

- **implemented-equivalent** — the same reader or governance responsibility is
  present with the same effective boundary.
- **implemented-different-by-design** — the responsibility exists, but Rust
  Protocol, the shared external Runtime, or an explicit Agent adapter owns it
  at a different path or abstraction.
- **migrate-gap** — a concrete responsibility has no accepted counterpart and
  needs a bounded remediation.
- **not-applicable** — the reference file is outside this Runtime's product
  boundary.
- **reference-only** — the file is retained as explanatory or conformance
  material, not as current Runtime behavior.
- **generated-history** — immutable reference history or generated projection;
  it is never copied or silently rewritten.
- **deferred-next-batch** — the path is recorded but its semantic comparison is
  intentionally scheduled for a later batch. This is not a claim of parity or
  omission.

## First batch: governance entrypoints

The first batch covers root Agent rules, `.ai` entrypoints and terminology,
reader-facing README and architecture routes, and the reference governance
configuration entrypoints. The Rust project keeps the important boundaries but
does not copy the reference's Python runtime, Makefile targets, YAML guard
tree, provider-global rules, or generated history.

| Reference surface | Rust result | Boundary |
| --- | --- | --- |
| `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, Cursor rule | Implemented differently | The repository uses an attached adapter and explicit provider installation. The shared Runtime remains external; no provider-global configuration is injected by comparison. |
| `.ai/README.md`, glossary, cockpit workflow/adoption guides | Implemented differently | `.ai/README.md`, `.ai/glossary.md`, `docs/reference/agent-workflow.*`, and the getting-started route carry the Rust request-scoped Runtime workflow. |
| Reference guards, policies, quality and trust schemas | Implemented differently | Typed Rust Protocol/Runtime services, repository tests, CI manifests, and reference documentation provide the corresponding controls. The source YAML/JSON files are not copied. |
| Root and documentation README routes | Implemented differently | The three language routes link to one another and describe shared Runtime plus isolated repository contexts. |
| `SECURITY.md` | Implemented equivalently with Rust-specific additions | The security boundary remains a policy entrypoint and includes the Runtime deployment/patch boundary. |
| `CONTRIBUTING.md` | Implemented in this batch | Contributor rules now describe the explicit `--repo` lifecycle, fail-closed evidence, visible Outcome, reviewed PR, and exact post-merge cleanup. |
| Reference generated Work Items, decisions, evidence, audits and release history | Generated-history | These bytes remain reference history and are not copied into the Rust repository. |

The first batch therefore closes the only concrete entrypoint gap found in the
baseline (`CONTRIBUTING.md`) without creating a second governance system. The
remaining paths are explicitly staged in the ledger for the next semantic
batches rather than silently treated as equivalent.

## WI-270 file-level Contract semantics slice

WI-270 compares the following 27 reference paths individually. The inventory
uses `implemented-different-by-design` for each path: the responsibility is
present in the Rust Runtime or its repository-bound documentation/tests, but
the Python module, Make target, generated file, and provider-global path are
not copied. The counterpart column is evidence, not a claim that the two
implementations have identical bytes.

| Reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/concepts/decision-states.ja.md` | implemented-different-by-design | `docs/reference/contract-fields.ja.md`, `docs/reference/outcome-report.ja.md`, typed decision/Outcome tests |
| `docs/concepts/decision-states.md` | implemented-different-by-design | `docs/reference/contract-fields.md`, `docs/reference/outcome-report.md`, typed decision/Outcome tests |
| `docs/concepts/decision-states.zh-CN.md` | implemented-different-by-design | `docs/reference/contract-fields.zh-CN.md`, `docs/reference/outcome-report.zh-CN.md`, typed decision/Outcome tests |
| `docs/features/work-item-parallelism.ja.md` | implemented-different-by-design | WI-123, Japanese configuration route, Contract boundary/lease tests |
| `docs/features/work-item-parallelism.md` | implemented-different-by-design | WI-123, configuration route, Contract boundary/lease tests |
| `docs/features/work-item-parallelism.zh-CN.md` | implemented-different-by-design | WI-123, Chinese configuration route, Contract boundary/lease tests |
| `docs/reference/safe-parallel-verification.md` | implemented-different-by-design | Rust bounded executor, `verify --workers`, argv-only execution and per-command evidence tests |
| `docs/reference/work-item-intelligence-interface.md` | implemented-different-by-design | Request-scoped Rust status/intelligence exists; full reference cost/wait/index-version aggregation remains a later projection boundary |
| `docs/reference/work-item-state-machine.md` | implemented-different-by-design | Typed lifecycle/recovery/finalization state machine; provider PR states are external evidence |
| `docs/reference/work-item-status-interface.md` | implemented-different-by-design | Rust status/Outcome projection and status tests replace the generated Python status file |
| `scripts/ai_acceptance_policy.py` | implemented-different-by-design | `governance_controls.rs` acceptance identifiers/evidence validation |
| `scripts/ai_check_scenario_coverage.py` | implemented-different-by-design | Runtime scenario coverage validation and Contract/Summary binding |
| `scripts/ai_check_work_item.py` | implemented-different-by-design | Typed Contract scope, authority, unknown, execution, concurrency, and lifecycle validation |
| `scripts/ai_decision_protocol.py` | implemented-different-by-design | Repository-bound typed preflight decision receipts |
| `scripts/ai_intent_policy.py` | implemented-different-by-design | Runtime intent alignment and intent/scenario binding |
| `scripts/ai_parallel_verification.py` | implemented-different-by-design | Rust bounded executor with worker caps, deterministic results, and scope safety |
| `scripts/ai_preflight_review.py` | implemented-different-by-design | Typed preflight state, humanDecisionRequest, confirmation, and recovery conditions |
| `scripts/ai_scenario_policy.py` | implemented-different-by-design | Risk-sensitive Runtime scenario policy and fail-closed unknowns |
| `scripts/ai_work_item_state.py` | implemented-different-by-design | Rust lifecycle state machine and recovery receipts |
| `tests/test_acceptance_policy.py` | implemented-different-by-design | Rust Contract schema/preflight regression tests |
| `tests/test_ai_parallel_verification.py` | implemented-different-by-design | Rust CLI/executor verification tests |
| `tests/test_checkpoint_intent.py` | implemented-different-by-design | Rust preflight/checkpoint intent tests |
| `tests/test_contract_and_policy.py` | implemented-different-by-design | Rust strict Contract and policy tests |
| `tests/test_intent_policy.py` | implemented-different-by-design | Rust intent alignment tests |
| `tests/test_parallel_lifecycle_contract.py` | implemented-different-by-design | Rust parallel boundary, lease, lifecycle, and isolation tests |
| `tests/test_preflight_review.py` | implemented-different-by-design | Rust preflight/review tests |
| `tests/test_scenario_coverage_gate.py` | implemented-different-by-design | Rust required-scenario and invalid-status tests |

The slice found no unrecorded implementation gap in these Contract semantics.
The intelligence-interface row is deliberately bounded: request-scoped status
and evidence-derived Outcome are implemented, while the reference's broader
aggregation and cost/wait dimensions remain scheduled and are not treated as
green parity.

## Current ledger snapshot

At the pinned v0.2.33 comparison baseline, the ledger contains 5,119 records:
4,262 `generated-history`, 176 `implemented-different-by-design`, one
`implemented-equivalent`, three `not-applicable`, and 677
`deferred-next-batch` records. Deferred records remain scheduled work, not
parity claims. The capability/profile slice has no remaining `migrate-gap`
records:

1. `.ai/project/adopter-capability-manifest.json` is represented by the
   Runtime registry and remains an external installer-surface boundary.
2. `.ai/project/capabilities.json` is represented by the strict Rust-native
   declaration and explicit operation mapping.
3. `.ai/project/success_criteria.json` is represented as a non-authoritative,
   snapshot-bound visibility projection.
4. `.ai/project_profile.yaml` is represented by `.ai/project.json` plus the
   strict JSON `profile-policy.json` projection.

The governance entrypoints, getting-started routes, CI/release boundaries, and
capability/profile projections have been reviewed at this baseline. The four
records above are Rust-native, explicitly bounded counterparts; the 677
deferred semantic comparisons remain scheduled work.

WI-274 rebinds only the target checkout metadata and canonical comparison
snapshot to the reviewed default-branch commit. WI-273 remains an immutable
failed-delivery record: its first commit could not prove that parity
registration preceded verification evidence, so the successor redelivery
keeps that history separate and does not rewrite it.

## Batch order

Later batches will compare and, where necessary, implement bounded differences
in this order:

1. Contract fields, intent, scenario/acceptance dimensions, parallel slots and
   preflight review.
2. CI quality routing, dynamic verification tiers, and evidence assurance.
3. Runtime lifecycle, Outcome/MCP projection, recovery, knowledge, and
   repository isolation.
4. Conformance, adversarial cases, performance, release, and adopter
   acceptance.

Each batch gets its own Contract and evidence. After a batch is reviewed and
published, the next batch is rechecked with the published Runtime so that a
working-tree change cannot masquerade as release behavior.

## WI-286 file-level Agent Risk and checkpoint slice

WI-286 compares the reference Agent Risk/checkpoint responsibility one file at
a time. Source Python/YAML remains reference corpus only; Rust typed Protocol
records and shared lifecycle validators enforce the bounded semantics.

| Reference path | Classification | Rust counterpart |
| --- | --- | --- |
| `.ai/guards/agent_risk_policy.yaml` | implemented-different-by-design | Typed `checkpointPolicy`, Contract verification declarations, Agent Risk validator, and dynamic profile docs. |
| `scripts/ai_check_agent_risk.py` | implemented-different-by-design | `validate_agent_risk_controls` is reused at lifecycle boundaries. |
| `scripts/ai_checkpoint.py` | implemented-different-by-design | Typed `CheckpointEvidence`, amendment CLI, append-only chain, and resume-stale binding. |
| `tests/test_ai_agent_risk.py`, `tests/test_ai_checkpoint.py`, `tests/test_outcome_lifecycle_rules.py` | implemented-different-by-design | Rust protocol/repository lifecycle and static Agent-rule parity tests. |

This is semantic, not direct JSON-wire parity. WI-291 adds the Rust read-only
Contract-aware CI gate and keeps the Python route/manifest as a shadow during
convergence; full workflow and release-preflight parity remains deferred.

## WI-287 checkpoint conformance closure

WI-287 closes the two ledger records that were still deferred for the source
checkpoint implementation and test corpus. The Rust side now explicitly
rejects a `before_edit` checkpoint after verification has started and rejects an
invalid latest resume timestamp. The source test behavior is represented by
Rust-native lifecycle regressions, not copied Python tests or source wire
shapes. The static Agent-rule test asserts the same terminality and narrow
successor boundary using the repository's own instructions.

The object/adopter boundary is unchanged: the installed shared Runtime is
request-scoped, every operation carries `--repo`, and human Outcome remains the
visible handoff. CI workflow convergence and broader adopter surfaces remain
separate bounded batches.

## WI-291 CI Contract-aware quality gate

WI-291 compares the reference workflow quality routing and preflight boundary
with the Rust-native CI surface. The Python route remains a dynamic planner for
`light`/`standard`/`strict` and the canonical manifest remains the command
list. Before standard or strict pull-request commands, the Rust CLI's read-only
`gate` validates the active Contract, repository/base/snapshot identity,
intent/scenario/operation/stage route, and Agent-Risk/preflight projection. It
emits an identity-bound `repository_contract_quality_gate` receipt; yellow or
red is a fail-closed CI result. The gate does not write `.ai/` records.

This batch is semantic parity, not source YAML or Python wire compatibility.
CI source-build Runtime identity is diagnostic; immutable Release/adopter
identity remains a separate published-artifact acceptance boundary. The
remaining reference workflow matrix, gate metadata/timeout model, release
preflight, and multi-stack adopter surfaces stay deferred and are recorded in
the inventory ledger rather than claimed as implemented.

## WI-302 first deferred file batch

WI-302 compared the first ten deferred paths in lexical order against the
pinned source commit. Eight records received an evidence-backed conclusion.
WI-304 then compared the two workflow records that contain the source's broad
Python/multi-stack matrix and recorded their Rust-native split and external
adopter boundary.

| Reference path | Classification | Rust counterpart / boundary |
| --- | --- | --- |
| `.ai/cockpit/bandit_low_risk_baseline.json` | not-applicable | Generated Bandit baseline for source Python tooling; no Rust/Bandit product surface. |
| `.gitattributes` | implemented-different-by-design | Rust source-archive boundary and `tests/release/source_archive_policy_test.sh` exclude governance/build roots while retaining Cargo sources. |
| `.github/CODEOWNERS` | not-applicable | Personal source owner is not portable; adopter review ownership is an external repository/provider decision documented in contributor/adopter guidance. |
| `.github/dependabot.yml` | not-applicable | Optional pip/Actions update automation is provider-owned; Rust dependency facts are `Cargo.toml`/`Cargo.lock` and pinned-action policy. |
| `.github/workflows/compatibility.yml` | implemented-different-by-design | WI-304 compares shellcheck, lockfile, Python, real/extended/mobile matrix, and non-blocking latest probes. Rust `ci.yml`, dynamic quality routing, canonical gates, and public adopter acceptance own the Rust product; source installer/Python/multi-stack coverage remains an explicit adopter/external boundary. |
| `.github/workflows/release.yml` | implemented-different-by-design | Rust release workflow and release tests provide target archives, checksums, SBOM/provenance, platform smoke, and public/N-1 adopter acceptance. |
| `.github/workflows/smoke.yml` | implemented-different-by-design | WI-304 compares every source shard, dispatch input, artifact, dependency edge, release/measurement condition, and installer check. Rust `ci.yml`, `release.yml`, gate manifest, and immutable adopter harnesses split those responsibilities; source Python/Make/install smoke remains external/adopter-owned. |
| `.gitignore` | implemented-different-by-design | Rust/Cargo build and governance review paths are ignored and source-archive policy is tested. |
| `LICENSE` | implemented-different-by-design | Both publish MIT; target-specific copyright and Rust packaging are intentionally not copied from the source. |
| `Makefile` | implemented-different-by-design | Rust CLI, Cargo, and explicit CI/release scripts replace source Python Make orchestration with request-scoped `--repo`. |

The WI-302/WI-304 batches found no `migrate-gap`. The inventory is now 4,262
`generated-history`, 176 `implemented-different-by-design`, one
`implemented-equivalent`, three `not-applicable`, and 677
`deferred-next-batch` records. The two workflow records are closed as
Rust-native, different-by-design boundaries; this does not claim that the
source's Python installer or multi-stack matrix runs inside the Rust Runtime.

## WI-304 workflow comparison

WI-304 compares `.github/workflows/compatibility.yml` and
`.github/workflows/smoke.yml` at the pinned source commit, including triggers,
permissions, concurrency, every job and matrix, `needs` edges, dispatch
inputs, artifact uploads/downloads, blocking versus non-blocking conditions,
release/measurement branches, and installer checks.

`compatibility.yml` has eight responsibilities: ShellCheck of the source
`install.sh`; pinned Python platform and lockfile reproducibility lanes;
real, extended, and mobile stack quality matrices; a non-blocking latest
ecosystem probe; and separate blocking/latest aggregate gates. The Rust
counterpart is intentionally split: `ci.yml` selects the repository's
dynamic `light`/`standard`/`strict` route and canonical gate manifest, while
Rust workspace/platform checks and the published adopter harness verify the
Runtime and repository Protocol. There is no target `install.sh`, Python
lockfile, or source Make orchestration. Toolchain and stack coverage for an
adopter is therefore configured and evidenced by that adopter or its hosted
provider; it is not silently claimed as product parity.

`smoke.yml` has project-test manifest/core/governance/installer/lifecycle/
release shards, a template aggregation job, installation smoke, conditional
release evidence, and a final CI evidence receipt. The Rust target assigns
the corresponding boundaries to `ci.yml` (Contract-aware quality, Windows,
and locked behavioral oracle), `release.yml` (archives, SBOM, checksums,
provenance, release policy), the canonical gate manifest, and strict public/
N-1 adopter acceptance. The source's Python test shards, `install.sh`/Make
smoke, and exploratory latest-toolchain probes have no target equivalent and
are explicitly external or adopter-owned.

This is semantic responsibility parity, not workflow-byte or source-command
parity. The target shell scripts currently have syntax validation; a
ShellCheck gate for target scripts is a separate CI-hygiene decision because
the source gate specifically checks a non-existent target installer. No
source Python module, Make target, installer, or multi-stack fixture is copied
by this batch.
