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

This page explains how the Rust project compares itself with the operator's
local reference source one file at a time. The reference is a specification
and behavior corpus; it is not a directory to copy into the Rust Runtime.

## Pinned baseline

- Current reference checkout: the local Git checkout supplied through
  `AI_COCKPIT_REFERENCE_ROOT`, pinned for current comparison work to
  `fde3380f81fea5fd2e288f7a8849f737dc074060` in
  `tests/conformance/reference-source.lock`.
- Rust comparison baseline: [`xinglun/ai-cockpit`](https://github.com/xinglun/ai-cockpit) `origin/main` at `cb8248fdf8ac8d965d8d8eb7b53760147bd13fcd`.
- Runtime used for the comparison work: `ai-cockpit 0.2.47`, binary SHA256 `6b3bd6617c6372a17b1edf6f9dc9dbc016779146f67262265fd12d2a488bbc53`.

The inventory ledger is now explicitly rebaselined to the local checkout. The
previous `e5acb677da6621004d96f0ef353c58fe8d3acfbf` ledger remains recoverable
from the recorded previous target revision and digest; it is not silently
rewritten. Source paths removed from the current checkout are listed in
`retiredReferencePaths`, and non-history paths whose source bytes changed are
marked `deferred-next-batch` with their previous decision retained as history.
Missing, dirty, or mismatched local source is fail-closed.

For audit continuity, the immediately previous comparison baseline remains
historical: target `bc8b7e56a98d105cd9f00b3b7300dc8eb0396c7b`, Runtime
`ai-cockpit 0.2.33`, binary digest
`eceed75ef74079e7ede420b42f8223fc76be82ec0211ddc6b8fdf7cb3c3b9de4`.

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

## WI-411 Java multi-module fixture boundary

WI-411 reads the nine files under `examples/fixtures/java-multimodule/` at the
pinned reference commit. They are all classified as `reference-only`. The
fixture demonstrates a Java application, an inter-module dependency, local
`javac`/`java` checks, and a disposable upgrade/rollback exercise. It is an
executable sample for the reference repository, not AI Cockpit Runtime code or
portable enterprise evidence.

| Reference path | Decision and target boundary |
| --- | --- |
| `.gitignore` | Fixture-local build hygiene; the target release harness owns its own isolated temporary roots. |
| `app/src/main/java/fixture/app/Main.java` | Java application sample; the target accepts adopter-declared argv but does not bundle Java support. |
| `app/src/test/java/fixture/app/MainTest.java` | Fixture assertion; a target verification receipt records an adopter command, not this source test. |
| `core/src/main/java/fixture/core/Decision.java` | Domain sample policy; repository policy remains explicit and typed, never copied from the fixture. |
| `core/src/test/java/fixture/core/DecisionTest.java` | Fixture-only test; it is not Runtime or enterprise evidence. |
| `evidence.json` | Source-local evidence with unavailable Maven/provider capabilities; target release receipts require stronger identity and isolation bindings. |
| `fixture.json` | Source stack/module metadata; target does not infer adopter capabilities from it. |
| `pom.xml` | Maven build input; Java/Maven execution remains an adopter or delegated-provider responsibility. |
| `scripts/lifecycle.sh` | Source fixture orchestration; target lifecycle is provided by the installed Rust Runtime and explicit repository-bound commands. |

No Java files, Maven manifests, or source shell orchestration are copied into
the Rust repository. A future second-technology adopter acceptance is a
separate, explicitly authorized Work Item; this comparison does not claim it.
The machine ledger and its regression test bind all nine paths to this
decision, so they cannot silently return to `deferred-next-batch`.

## WI-414 Python fixture boundary

WI-414 reads the four files under `examples/fixtures/python/` at the pinned
reference commit. They are all `reference-only`: the fixture demonstrates a
Python service, packaging metadata, and a pytest assertion, but it is not Rust
Runtime code, a Python toolchain promise, or portable enterprise evidence.

| Reference path | Decision and target boundary |
| --- | --- |
| `fixture.json` | Sample stack, platform, and path metadata; the target keeps adopter facts repository-local and does not infer Python capability from this file. |
| `pyproject.toml` | Sample packaging and pytest configuration; Python installation and test commands remain adopter/provider responsibilities. |
| `src/service.py` | Application sample returning `ok`; it is not governance logic and is not copied into the target. |
| `tests/test_service.py` | Fixture-only pytest assertion; it is not Runtime or enterprise evidence, and the adopter must declare its own verification command. |

No Python source, dependency manifest, installer, or test runner is copied into
the Rust repository. The attached shared Runtime still provides the same
Contract, evidence, lifecycle, and human Outcome controls to a Python adopter,
but this is semantic/documentation parity rather than Python toolchain or
source-command compatibility. The machine ledger and regression test bind all
four paths to this boundary, so they cannot silently return to
`deferred-next-batch`.

## WI-432 TypeScript web fixture boundary

WI-432 reads the eleven files under `examples/fixtures/typescript-web/` at the
pinned reference commit. They are all `reference-only`: the fixture demonstrates
a TypeScript application, npm tooling, local format/lint/test checks, and a
sample lifecycle script, but it is not Rust Runtime code, a Node toolchain
promise, or portable provider/enterprise evidence.

| Reference path | Decision and target boundary |
| --- | --- |
| `.gitignore` | Fixture-local build hygiene; the target release harness owns its own isolated roots. |
| `evidence.json` | Source-local npm evidence and unavailable provider claims; target receipts require explicit commands and identity binding. |
| `fixture.json` | TypeScript/web stack and path metadata; the Runtime does not infer adopter capabilities or Contract scope from it. |
| `package-lock.json` | Adopter-owned npm dependency lock; it is not a Runtime dependency or release proof. |
| `package.json` | Application build/test/lint/format/lifecycle scripts; adopters declare explicit argv while governance lifecycle remains Runtime-owned. |
| `scripts/format-check.mjs` | Fixture-specific formatting rule, not a portable governance control. |
| `scripts/lifecycle.mjs` | Node install/configure/block/upgrade/rollback/release exercise; Runtime governance and recovery are not copied from it. |
| `scripts/lint.mjs` | Application-specific lint rule; adopters own their lint command and evidence. |
| `src/index.ts` | Sample application evaluator; Runtime does not import or infer its policy. |
| `test/index.test.mjs` | Fixture-only Node tests; adopters must declare and run their own verification. |
| `tsconfig.json` | Adopter-owned strict TypeScript compiler configuration; no Node/TypeScript toolchain is promised. |

No TypeScript source, npm dependency, installer, or Node lifecycle script is
copied into the Rust repository. An attached TypeScript/web adopter inherits
the shared Contract, fail-closed evidence, repository isolation, lifecycle, and
human Outcome controls, but this is semantic/documentation parity rather than
TypeScript toolchain or source-command compatibility. The machine ledger and
regression test bind all eleven paths to this boundary.

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

<!-- reference-inventory-counts: total=4450 generated-history=3681 implemented-different-by-design=230 implemented-equivalent=1 not-applicable=4 reference-only=62 deferred-next-batch=472 migrate-gap=0 -->

At the current local reference comparison baseline, the ledger contains 4,450
current tracked paths: 3,681 `generated-history`, 230
`implemented-different-by-design`, one `implemented-equivalent`, four
`not-applicable`, 62 `reference-only`, and 472 `deferred-next-batch` records.
Deferred records remain scheduled work, not parity claims. The rebaseline also
records 160 changed current paths (143 non-history decisions awaiting a fresh
semantic comparison) and 669 retired paths from the previous ledger. The
capability/profile slice has no remaining `migrate-gap` records:

1. `.ai/project/adopter-capability-manifest.json` is retired from the current
   local checkout. Its prior decision is retained in `retiredReferencePaths`;
   it remains an external installer-surface boundary, not a current record.
2. `.ai/project/capabilities.json` is represented by the strict Rust-native
   declaration and explicit operation mapping.
3. `.ai/project/success_criteria.json` is represented as a non-authoritative,
   snapshot-bound visibility projection.
4. `.ai/project_profile.yaml` is represented by `.ai/project.json` plus the
   strict JSON `profile-policy.json` projection.

The governance entrypoints, getting-started routes, CI/release boundaries, and
capability/profile projections retain their prior evidence-backed decisions
where the source bytes are unchanged. Changed paths are deliberately deferred
until a later file-by-file batch re-reads the local source; retired paths are
historical only. The three current capability/profile records above remain
explicitly bounded Rust-native counterparts; the fourth path is historical only.

## WI-435 local-reference rebaseline

WI-435 binds the active ledger to the operator-maintained checkout at
`fde3380f81fea5fd2e288f7a8849f737dc074060` without treating a source update as
a semantic comparison. The current manifest contains the exact tracked path
set and records the previous source commit, previous manifest digest, changed
paths, and retired paths. A changed non-history record is intentionally
`deferred-next-batch` until a later Work Item reads that file again; a removed
path is retained as historical metadata and does not become an invisible
omission. This keeps the public reference repository out of the comparison
path while preserving auditability across local source updates.

WI-274 rebinds only the target checkout metadata and canonical comparison
snapshot to the reviewed default-branch commit. WI-273 remains an immutable
failed-delivery record: its first commit could not prove that parity
registration preceded verification evidence, so the successor redelivery
keeps that history separate and does not rewrite it.

## WI-437 local-reference governance rebaseline delta

WI-437 re-reads the seven governance files whose bytes changed between the
previous public-reference ledger and the operator-maintained local checkout.
All seven are `implemented-different-by-design`; none introduces a Rust
Runtime omission or a requirement to copy source artifacts.

| Local reference path | Rust result | File-level decision |
| --- | --- | --- |
| `.ai/cockpit/README.md` | Implemented differently | The source removed a Python-template Implementation Approach section. Rust keeps its evidence-bound approach and Outcome projection in typed Runtime/docs surfaces. |
| `.ai/cockpit/README.ja.md` | Implemented differently | The source removed the obsolete `REPORT_LANGUAGE` Make argument; Runtime-owned presentation localization already covers this boundary. |
| `.ai/cockpit/adoption.ja.md` | Implemented differently | The source onboarding example no longer passes `REPORT_LANGUAGE`; Rust onboarding has no template-local Make command and uses explicit `--repo`. |
| `.ai/guards/changed_critical_coverage_policy.json` | Implemented differently | Removed Python-only coverage associations are represented by native tests, governance integrity, and typed Runtime controls. |
| `.ai/guards/coverage_policy.yaml` | Implemented differently | The source association registry is not a Rust configuration surface; coverage ownership is expressed by native tests and CI gate manifests. |
| `.ai/quality/governance-routing.yaml` | Implemented differently | The source separates route selection from duplicated depth/evidence fields; Rust preserves the same separation through dynamic routing and the versioned gate manifest. |
| `.ai/schemas/task_outcome.schema.json` | Implemented differently | The source simplified its Python Task Outcome schema. Rust `OutcomeV2`/`humanHandoff` is a separate typed Protocol/presentation contract and is not removed or copied from the source schema. |

The source diff was therefore a reference-side cleanup of Python/Make
surfaces, not a portable feature delta. The ledger retains the source-change
provenance (`previousBatch`, `previousClassification`, and
`sourceChangedSincePrevious`) while the seven current records are no longer
deferred. This explicit result prevents the local rebaseline from reopening
the same files in every future comparison.

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

At the completion of the WI-302/WI-304 batches, the inventory snapshot was 4,262
`generated-history`, 190 `implemented-different-by-design`, one
`implemented-equivalent`, three `not-applicable`, three `reference-only`, and 660
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

## WI-305 architecture installation and verification slice

WI-305 compares the next four deferred reference files individually at the
pinned commit. They describe a read-only installation detector, an optional
ten-stage interactive Installer Wizard, stage-aware lightweight verification,
and Wizard input/localization primitives. The target is intentionally not a
byte-compatible copy of those Python adapters. Each responsibility is mapped
to the shared Rust Runtime, an explicit adopter boundary, or reference-only
material below.

| Reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/architecture/installation-detection-boundary.md` | implemented-different-by-design | `inspect`, `status`, `doctor`, `attach`, `profile propose`, first-calibration docs, and CLI attach/profile tests provide read-only facts and explicit write boundaries. Immutable Release installation is separate from repository onboarding. |
| `docs/architecture/interactive-installation-wizard.md` | reference-only | The source ten-stage wizard, dry-run Installer preview, and confirmation UI are not a Rust Runtime feature. The supported target route is public Release verification followed by explicit `inspect` → `attach` → profile review/confirmation → `doctor`; an Agent adapter may supply conversation UX but cannot create approval. |
| `docs/architecture/lightweight-verification-and-soft-gates.md` | implemented-different-by-design | Typed stages, policy-driven tiers, fail-closed governance decisions, skipped/unknown reasons, one request-scoped context, dynamic `light`/`standard`/`strict` routing, and advisory cost/reuse telemetry are covered by the verification route, CI gate, and cost-observation tests. The source `hard`/`soft`/`informational` checker labels are an explicit documented boundary, not a copied generic wire enum; source Make/Python checker orchestration is not copied. |
| `docs/architecture/wizard-io-and-localization.md` | implemented-different-by-design | CLI/MCP human Outcome and command presentation localize `en`/`zh-CN`/`ja`, preserve contract values verbatim, and fail closed on explicit command/preflight boundaries. Wizard-specific TTY back/pause/help controls are not shipped because the target has no interactive Installer Wizard; adapters own conversation controls. |

### File-level findings and migration boundary

The source detector's `new_adoption`/`upgrade` distinction maps to the target's
separate Release installation and repository-local attach/profile decisions.
Target inspection is read-only; `attach` and profile confirmation are explicit
repository writes, and no command infers authority from prose or a detected
stack. Active Work Items, dirty state, conflicts, symlink risks, and missing
facts remain reasons to stop or request review rather than reasons to guess.

The source interactive wizard is a convenience layer around its Python
Installer, not a requirement to copy an Installer into this Rust repository.
Its ten steps, dry-run preview, cancellation, rollback boundary, and no-PR/no-
merge promise are preserved as adopter-facing boundaries in the installation
route. The target does not expose a second transaction authority or an
interactive prompt that could bypass Contract/preflight/human-decision rules.

The source soft-gate document's `hard`, `soft`, and `informational` distinction
is not copied as a generic target wire enum. The target maps the same safety
boundary to fail-closed governance decision states plus explicitly advisory
observations; a stage-inapplicable check is explicit rather than omitted,
trend/cost observations remain advisory, and `pre_ci` never becomes hosted CI
evidence. The selected tier and assurance are policy-bound, not inferred from
execution speed. The same rule applies in an adopter repository: the shared
Runtime is external, `--repo` is explicit, and provider or enterprise controls
remain delegated evidence.

Localization is presentation-only. Runtime-generated headings, status,
unknowns, recovery text, and next actions can follow the configured language;
paths, commands, Contract intent, acceptance criteria, and machine evidence
remain their authored values. This is not a claim that the Runtime provides
general translation or a source-compatible Wizard UI. No `migrate-gap` was
found in this slice; the interactive wizard itself remains an explicit
reference-only boundary rather than an unrecorded omission.

## WI-308 reference evidence, trust, and rollback-corruption slice

WI-308 compares four files at the pinned source commit `e5acb677`: one visual
demo asset, a hypothetical rollback-corruption case study, Evidence Governance,
and the Trust Layer. The target records each file separately and keeps the
source implementation and binary assets out of the Rust repository.

| Reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/assets/ai-cockpit-demo.gif` | reference-only | The pinned GIF is GIF89a, 800x435, 587,945 bytes, SHA-256 `88838de7221dc859efde7e8e87913d0a23a21466195647ded60612adbad1f795`. It remains a source visual reference; no binary copy or Runtime contract is claimed. |
| `docs/case-study-ai-rollback-corruption.md` | implemented-different-by-design | Tri-language adversarial-validation docs plus typed Contract/scope checks cover unauthorized paths, unrelated changes, and controlled recovery. The case remains hypothetical and the Runtime does not auto-rollback, approve a merge, or infer business impact. |
| `docs/concepts/evidence-governance.md` | implemented-different-by-design | `docs/security/enterprise-governance.*`, `docs/reference/outcome-report.md`, and typed Protocol/Repository evidence project the Evidence → Governance Decision → Human Control chain. Provider evidence remains delegated and prose is never proof. |
| `docs/concepts/trust-layer.md` | implemented-different-by-design | `docs/architecture/product-boundary.md`, `docs/philosophy.md`, enterprise-governance, and the Runtime capability truth registry define calibrated trust, fail-closed unknowns, human control, and explicit non-goals. The source public claim matrix is not a target gate. |

The migration is semantic responsibility parity, not source wire or byte
compatibility. The target's richer Contract/evidence schemas and shared
request-scoped Runtime preserve the source safety intent while adding explicit
repository identity, snapshots, human decisions, and provider boundaries. The
GIF is deliberately reference-only. No source Python, Make, installer, or
binary is copied, and no local evidence is promoted to provider or enterprise
assurance. The same conclusions and reader route are available in Chinese and
Japanese.

## WI-323 reference documentation foundation

WI-323 compares the next nine deferred documentation paths individually at the
pinned source commit. The batch closes documentation responsibilities without
copying source tooling or changing Runtime authority.

| Reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/contributing/installation-document-maintenance.md` | implemented-different-by-design | Tri-language reference routes and documentation acceptance scripts preserve the thin-home, link/metadata, version-neutral, no-guess/no-overwrite/no-fallback, and separate-approval boundaries. |
| `docs/current/README.md` | implemented-different-by-design | `docs/current/README.*`, `.ai/README.md`, `.ai/glossary.md`, `AGENTS.md`, and `docs/reference/README.*` form the current Agent read route. The source `make ai-documentation-read-set` is not a target command. |
| `docs/design/harden-work-item-pr-closure.md` | implemented-different-by-design | `docs/reference/agent-workflow.*`, `docs/reference/commands.md`, and the Rust lifecycle enforce latest-base, dedicated branch, reviewed PR, merge-before-close, synchronization, and exact cleanup; provider PR operations stay external. |
| `docs/distribution.md` | implemented-different-by-design | The target's current route and `docs/release/distribution.*` provide the compatibility entry, immutable artifact installation, and post-release adopter boundary. |
| `docs/enterprise-security-boundary.md` | implemented-different-by-design | `docs/security/enterprise-deployment-boundary.*`, `enterprise-governance.*`, and `SECURITY.md` separate repository evidence from delegated identity, sandbox, audit, and certification controls. |
| `docs/examples/trust-layer-demo.sh` | reference-only | The offline stop/continue examples remain explanatory source material; typed Runtime preflight, capability, intent, and adversarial tests are the target evidence, not a copied shell authority. |
| `docs/features/human-benefit-report.md` | implemented-different-by-design | Rust `OutcomeV2`, `work-item outcome`, MCP `work_item_outcome`, and tri-language handoff tests preserve the human report order and evidence boundaries. |
| `docs/features/human-benefit-report.zh-CN.md` | implemented-different-by-design | Chinese presentation follows the same Rust Outcome/MCP route; Contract acceptance text remains authored and is not machine-translated. |
| `docs/features/human-benefit-report.ja.md` | implemented-different-by-design | Japanese presentation follows the same Rust Outcome/MCP route; Contract acceptance text remains authored and is not machine-translated. |

The Cursor adopter feedback is consistent with this boundary after version
normalization: current Runtime lifecycle commands already emit stable stdout
JSON and human handoff, `work-item new`/`start` reject unclosed archives and
pre-existing changes, and readiness is explicit. A CLI cannot force Cursor to
expand a chat panel; the provider/Agent adapter must surface or replay the
human handoff. Diagnostic remediation, close-gap convenience commands, and
optional controls scaffolding are follow-up product decisions, not silently
claimed parity in this documentation batch. The target also deliberately has
no `Makefile.ai` requirement: explicit `--repo` CLI/MCP commands are the
repository-neutral adopter interface.

This batch is semantic responsibility parity, not source wire or byte parity.
Source Make/Python report generators, installer scripts, and the trust demo
are not copied. The object-engineering boundary is the same as for every
adopter: one shared external Runtime, repository-local `.ai/` state, explicit
repository context, and provider-owned conversation presentation.

## WI-326 reference quality, overview, philosophy, and closure-plan slice

WI-326 compares the following nine pinned reference paths individually. Eight
are implemented differently by design; the closure hardening plan is retained
as reference-only because it is an internal historical plan, not a current
Runtime command contract.

| Reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/non-make-adaptation.ja.md` | implemented-different-by-design | The installation and Agent workflow routes provide the external Runtime and repository-local adapter boundary. Adopter-owned stack commands remain outside Core; the source `Makefile.ai` bridge is not copied or required. |
| `docs/operations/quality-gates.ja.md` | implemented-different-by-design | The Japanese CI quality-gate and manifest routes preserve gate ownership, evidence, traceability, and policy-selected `light`/`standard`/`strict` routing. Source Make targets, Python checker registries, and template-maintenance fixtures are not copied. |
| `docs/operations/quality-gates.md` | implemented-different-by-design | The versioned Rust-native gate manifest and CI route preserve the source quality-gate semantics while keeping hosted CI and adopter stack checks at their owner boundary. |
| `docs/operations/quality-gates.zh-CN.md` | implemented-different-by-design | The Chinese quality-gate and manifest routes preserve the same evidence and dynamic-routing boundary; source Make/Python orchestration is not a target command. |
| `docs/overview.ja.md` | implemented-different-by-design | Rust architecture, capabilities, Agent workflow, and command routes preserve the source five-layer overview with request-scoped, repository-bound governance; source status/verification registries are not copied. |
| `docs/philosophy/design-philosophy.ja.md` | implemented-different-by-design | Japanese product-boundary, capability, and enterprise-governance docs preserve calibrated trust, evidence over self-declaration, proportional control, and human responsibility. |
| `docs/philosophy/design-philosophy.md` | implemented-different-by-design | English product-boundary, capability, and enterprise-governance docs preserve the same principles; Core is not an Agent Runtime, sandbox, identity provider, or compliance certificate. |
| `docs/philosophy/design-philosophy.zh-CN.md` | implemented-different-by-design | Chinese product-boundary, capability, and enterprise-governance docs preserve the same principles and explicit non-goals. |
| `docs/plans/harden-work-item-pr-closure.md` | reference-only | The source file is an internal historical Python `ai-finish`/`ai-close` hardening plan. Current Rust lifecycle and governance-integrity routes preserve its closure intent, but obsolete implementation steps and command names are not current capability. |

This batch found no `migrate-gap` record. The semantic boundary is preserved
without source wire or byte compatibility: quality decisions are made by the
versioned manifest and current Runtime, while provider-hosted checks, adopter
stack commands, and enterprise controls remain delegated. Dynamic routing is
policy-selected; a stricter tier is not inferred merely from execution speed,
and a tier is not an assurance level. The same boundary applies to an object
engineering repository using the published Runtime with an explicit `--repo`.

## WI-327 adopter, calibration, and long-cycle documentation slice

WI-327 compares the next nine deferred reference paths individually at the
pinned source commit. Eight are implemented differently by design; the
scanner-specific Bandit audit remains reference-only because its findings and
digest belong to the source Python toolchain.

| Reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/adopter-long-cycle-validation.ja.md` | implemented-different-by-design | The published-binary adopter and upgrade acceptance harness, distribution route, and Japanese lifecycle/security docs preserve isolated install, lifecycle, rollback, and cleanup evidence. Source multi-stack fixtures and Make/Python orchestration are not copied. |
| `docs/reference/adopter-long-cycle-validation.md` | implemented-different-by-design | The published-binary adopter and upgrade acceptance harness, distribution route, and lifecycle/security docs preserve isolated install, lifecycle, rollback, and cleanup evidence. Source multi-stack fixtures and Make/Python orchestration are not copied. |
| `docs/reference/adoption-reality-report.md` | implemented-different-by-design | Runtime capability/profile/status projections and the immutable adopter acceptance receipt preserve the distinction between template capability, adopter execution, provider evidence, and enterprise assurance. No local file is promoted to external proof. |
| `docs/reference/bandit-synchronization-security-audit.md` | reference-only | This is a source-specific historical Bandit finding inventory. The target has no Python/Bandit surface and does not claim the source count or digest; Rust-native quality and threat-model boundaries remain separately documented. |
| `docs/reference/calibration-inventory.md` | implemented-different-by-design | Repository-bound profile proposal/confirmation, capability/status projections, and explicit unknowns preserve the fact/evidence boundary without copying the source ten-column Python inventory. |
| `docs/reference/calibration-profiles.ja.md` | implemented-different-by-design | Japanese calibration guidance and strict JSON profile policy preserve cumulative Lite/Standard/Strict controls, human selection, monotonic upgrades, and explicit downgrade evidence; calibration remains separate from per-Work-Item quality routing. |
| `docs/reference/calibration-profiles.md` | implemented-different-by-design | Calibration guidance and strict JSON profile policy preserve cumulative Lite/Standard/Strict controls, human selection, monotonic upgrades, and explicit downgrade evidence; calibration remains separate from per-Work-Item quality routing. |
| `docs/reference/calibration-profiles.zh-CN.md` | implemented-different-by-design | Chinese calibration guidance and strict JSON profile policy preserve cumulative Lite/Standard/Strict controls, human selection, monotonic upgrades, and explicit downgrade evidence; calibration remains separate from per-Work-Item quality routing. |
| `docs/reference/calibration-session-model.ja.md` | implemented-different-by-design | The target keeps calibration proposal, confirmation, and repository-bound facts explicit. It does not silently introduce a generic interactive Session or checklist authority; unknowns and human responsibility remain visible. |

The comparison is semantic responsibility parity, not source wire or command
parity. The target uses one shared external Runtime and repository-local `.ai/`
state with explicit `--repo`; provider identity, hosted CI, signing, SBOM,
provenance, and enterprise controls remain delegated evidence. A Cursor adopter
must explicitly install its repository-local adapter and replay the durable
`work-item outcome` handoff; the Runtime cannot force an IDE chat panel to
expand. Current Runtime output and lifecycle entry gates are therefore not a
claim of automatic chat posting. Diagnostic remediation, close-gap convenience
commands, and automatic controls scaffolding remain separate product decisions.

## WI-328 calibration and capability-truth slice

WI-328 compares the next nine pinned reference paths individually. Five are
implemented differently by design; four capability-matrix/claim-authoring
documents remain explicit reference-only boundaries because the Rust target
does not ship the source public claim checker or matrix.

| Reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/calibration-session-model.md` | implemented-different-by-design | Repository-bound profile proposal, confirmation, and calibration facts preserve the source fact/evidence boundary; no generic persisted Session is introduced. |
| `docs/reference/calibration-session-model.zh-CN.md` | implemented-different-by-design | Chinese calibration/profile routes preserve the same explicit proposal, confirmation, unknown, and human-authority boundary. |
| `docs/reference/calibration-session.ja.md` | implemented-different-by-design | The source ten-stage Session is represented only by the target's explicit profile proposal/confirmation route; source Make/Python orchestration is not copied. |
| `docs/reference/calibration-session.md` | implemented-different-by-design | The source persisted wizard is source-specific orchestration. Target calibration is read-only-first and repository-bound, with human confirmation for policy changes. |
| `docs/reference/canonical-terminology.md` | implemented-different-by-design | .ai/glossary.md, configuration, and Outcome references provide canonical terms; governance light is not calibration lite, and release is an operation, not a profile. |
| `docs/reference/capability-claim-authoring.md` | reference-only | The source lexical claim checker and matrix front matter are not a target Runtime gate. The target registry reports observed facts and exclusions; candidate WI-330 would own any strict claim/evidence binding. |
| `docs/reference/capability-evidence-freshness.md` | reference-only | Work Item verification freshness exists, but no separate Capability Truth row expiry/portable-environment matrix is shipped; candidate WI-330 owns that scope. |
| `docs/reference/capability-truth-matrix.json` | reference-only | The source 30-row public matrix is not copied. capability_truth_registry is an observed-capability projection, not public claim authorization or adopter/provider proof. |
| `docs/reference/capability-truth-matrix.md` | reference-only | Current capability/adoption pages state observed-fact, adopter, provider, and enterprise boundaries; no source matrix or checker is claimed. |

The reference-only results are explicit product boundaries, not hidden
omissions. WI-330 closes this comparison by documenting that the source claim
checker, row-freshness matrix, and public matrix are not target Runtime
features. A future Rust-native claim/evidence gate remains optional and would
require a separately human-owned scope; no source Python/V1 asset is silently
promoted.

Cursor adopter feedback is external validation input. Current Runtime lifecycle
JSON, replayable work-item outcome, close-before-next/readiness checks, and
fail-closed start/verification bindings are already documented and tested.
Cursor/host adapters must surface the durable handoff because the Runtime
cannot expand an IDE chat panel. Diagnostic remediation, controls scaffolding,
close-gap convenience, and Makefile integration remain explicit non-goals here.

### Cursor adopter feedback assessment (v0.2.33)

This adopter matrix records current guarantees and explicit boundaries; it is
not a source wire-compatibility claim.

| Feedback | Current boundary | Decision |
| --- | --- | --- |
| Agent-facing Outcome output | `finish`, `archive`, and `close` emit stable lifecycle JSON on stdout; `work-item outcome --json` and repository-bound MCP `work_item_outcome` are replayable machine entrypoints. | Implemented in Runtime. Cursor must surface the handoff in chat; the CLI cannot expand an IDE panel. |
| Close before the next Work Item | Readiness/lifecycle entry rejects active Work Items, unclosed archives, dirty source paths, detached heads, and unsynchronized default bases. | Implemented fail-closed; `ready_on_base` is explicit. |
| Start timing and base binding | Start rejects pre-existing non-governance changes and binds explicit branch/worktree/base context before implementation. | Implemented fail-closed. |
| Finalize/close diagnostics | Errors include the failing boundary and recovery condition; there is no dedicated `close-gap` remediation command. | Partial; richer diagnostics are a future bounded product decision. |
| Controls scaffolding | Declared controls/evidence are validated; acceptance decisions are never invented and a complete controls template is not generated. | Deliberately decision-free. |
| Post-merge close recovery | Explicit `finalize`, `finalize-verify`, `close`, and readiness/status projections cover the lifecycle. | Current lifecycle is authoritative; a `close-gap` alias is optional host UX. |
| Make integration | The target uses explicit `--repo` CLI/MCP and provider adapters; source `Makefile.ai` orchestration is not a protocol requirement. | Not a parity omission; source Make/Python orchestration is not copied. |
| Verification invalidation | Source snapshot, Contract, repository identity, and evidence bindings are checked at lifecycle boundaries; source changes require fresh verification. | Implemented fail-closed; archived bytes remain immutable historical truth. |

Any future Runtime change must use a human-owned bounded Contract, tests,
tri-language documentation, and published-Runtime acceptance; adopter feedback
does not become an untracked promise.

## WI-330 capability-truth boundary decision

WI-330 re-reads the four pinned source files individually and records the
following final decision. The target's `capability show` projection remains
repository- and snapshot-bound, while public claim authorization and
Capability Truth row expiry are intentionally outside the current Runtime.

| Pinned source path | Final classification | Decision and target counterpart |
| --- | --- | --- |
| `docs/reference/capability-claim-authoring.md` | reference-only | The source lexical trigger/claim-binding checker is not copied. Documentation metadata does not grant evidence; public wording must rely on current bounded evidence and limitations. Counterparts: `docs/capabilities.md`, `crates/cockpit-repository/src/lib.rs`. |
| `docs/reference/capability-evidence-freshness.md` | reference-only | Work Item receipt freshness exists, but source Capability Truth row expiry and portable-environment policy do not. Counterparts: Runtime evidence validation and `docs/reference/outcome-report.md`. |
| `docs/reference/capability-truth-matrix.json` | reference-only | The source 30-row matrix is not a Rust wire format or authorization source. `capability_truth_registry` reports observed facts, adopter state, and external exclusions only. Counterparts: `crates/cockpit-protocol/src/lib.rs`, `crates/cockpit-repository/src/lib.rs`. |
| `docs/reference/capability-truth-matrix.md` | reference-only | The target capability/adoption pages state the observed/evidence/provider/enterprise boundary without advertising the source matrix or checker. Counterpart: `docs/capabilities.md`. |

This is a product-boundary decision, not an untracked omission. If a future
human-owned Work Item introduces claim binding or row freshness, it must define
Rust-native schemas, evidence generation, stale handling, multilingual scope,
and adopter acceptance before changing any classification.

## WI-331 checks catalog and CI/release evidence

WI-331 compares the next two pinned reference paths individually. Both are
implemented differently by design: the Rust target preserves the source
quality/release evidence responsibilities without copying the source Make,
Python, or V1 runtime.

| Pinned source path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/checks-catalog.md` | implemented-different-by-design | `docs/reference/checks-catalog.*`, the Contract-aware `gate` route, repository gate manifest, Rust workspace checks, conformance/docs checks, and release/adopter checks provide the same layered quality intent. Local checks remain distinct from provider or enterprise assurance; dynamic light/standard/strict profiles escalate on unknown or release-owned controls. |
| `docs/reference/ci-release-evidence.md` | implemented-different-by-design | `docs/reference/ci-release-evidence.*`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`, release distribution checks, and adopter acceptance harnesses bind provider jobs, commit/base/head, artifacts, checksums, SBOM, provenance, and isolation receipts. Skipped or failed jobs remain visible, and PR prose never becomes evidence. |

The semantic boundary is explicit. The target Runtime owns repository-local
Contract and gate decisions; hosted CI, signing, SBOM/provenance providers, and
enterprise audit systems own their delegated evidence. Public release truth is
bound to immutable tags and downloaded artifacts. `--repo` remains mandatory,
and a source `Makefile`, Python runner, or copied V1 runtime is not a target
requirement. The six language counterparts and inventory assertions are the
anti-omission record for this batch.

## WI-332 P0 comprehension-review evidence

WI-332 reads the three pinned comprehension-review evidence files individually.
All three remain `reference-only`: they are historical desk-review records
whose reviewer, date, score, and language claims belong to the reference
repository and cannot be transferred as evidence for this target. The target
does preserve the six-question reader route through its localized home,
philosophy, architecture, and Agent-workflow pages, with link and metadata
checks. It does not invent an independent native-language editorial review or
copy source evidence bytes. This is semantic reader alignment, not a claim that
the target has passed the source study.

| Pinned source path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/comprehension-review-2026-08-14.md` | reference-only | `docs/README.md`, `docs/philosophy.md`, `docs/architecture.md`, `docs/reference/agent-workflow.md`, and `tests/docs/documentation_acceptance.sh` provide the English reader route and structural checks; the source reviewer result is not portable evidence. |
| `docs/reference/comprehension-review-2026-08-14.zh-CN.md` | reference-only | `docs/README.zh-CN.md`, `docs/philosophy.zh-CN.md`, `docs/architecture.zh-CN.md`, `docs/reference/agent-workflow.zh-CN.md`, and the documentation acceptance checks provide the Chinese route; no native reviewer score is claimed. |
| `docs/reference/comprehension-review-2026-08-14.ja.md` | reference-only | `docs/README.ja.md`, `docs/philosophy.ja.md`, `docs/architecture.ja.md`, `docs/reference/agent-workflow.ja.md`, and the documentation acceptance checks provide the Japanese route; no native reviewer score is claimed. |

The external Cursor adopter feedback remains a separate validation input. The
Runtime's stable lifecycle JSON, replayable human Outcome, readiness/start
gates, and verification invalidation are already covered elsewhere. Automatic
Cursor chat posting, `Makefile.ai`, close-gap convenience commands, and
controls templates are not silently promoted to current parity by this evidence
batch.

## WI-333 comprehension-validation protocol and participant records

WI-333 reads the pinned comprehension-validation protocol, strict response schema,
six anonymized response records, and the bounded result files individually. All
twelve paths are `reference-only`. They describe an external human-reader study
owned by the reference repository; participant responses and source revision
claims are not portable evidence for this target. The target keeps its reader
documentation route and Runtime evidence checks separate from participant
research. No response bytes or source result is copied, and this repository makes
no comprehension, release, safety, security, or enterprise claim from the source
study.

| Pinned source path | Classification | Target counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/comprehension-validation-protocol.md` | reference-only | `docs/README.md`, `docs/reference/agent-workflow.md`, `docs/reference/outcome-report.md`; source eligibility, consent, interview and reviewer protocol remain external. |
| `docs/reference/comprehension-validation-protocol.zh-CN.md` | reference-only | `docs/README.zh-CN.md`, `docs/reference/agent-workflow.zh-CN.md`, `docs/reference/outcome-report.zh-CN.md`; no target participant study is implied. |
| `docs/reference/comprehension-validation-protocol.ja.md` | reference-only | `docs/README.ja.md`, `docs/reference/agent-workflow.ja.md`, `docs/reference/outcome-report.ja.md`; source ethics and eligibility are not Runtime policy. |
| `docs/reference/comprehension-validation-response.schema.json` | reference-only | `.ai/README.md`, `docs/reference/outcome-report.md`; the participant-response schema is not the Runtime Contract or verification-evidence schema. |
| `docs/reference/comprehension-validation-responses/peter_01.en.json` | reference-only | `docs/README.md`, `docs/features/human-benefit-report.md`; historical source response, revision and pseudonym stay source-bound. |
| `docs/reference/comprehension-validation-responses/peter_02.en.json` | reference-only | `docs/README.md`, `docs/features/human-benefit-report.md`; no participant data is imported into `.ai/`. |
| `docs/reference/comprehension-validation-responses/tanaka_01.ja.json` | reference-only | `docs/README.ja.md`, `docs/features/human-benefit-report.ja.md`; source response is not adopter or Runtime evidence. |
| `docs/reference/comprehension-validation-responses/tanaka_02.ja.json` | reference-only | `docs/README.ja.md`, `docs/features/human-benefit-report.ja.md`; source revision-bound facts remain external. |
| `docs/reference/comprehension-validation-responses/xiaoli_01.zh-CN.json` | reference-only | `docs/README.zh-CN.md`, `docs/features/human-benefit-report.zh-CN.md`; no native-language score is claimed for this target. |
| `docs/reference/comprehension-validation-responses/xiaoli_02.zh-CN.json` | reference-only | `docs/README.zh-CN.md`, `docs/features/human-benefit-report.zh-CN.md`; raw participant text is not copied. |
| `docs/reference/comprehension-validation-results.json` | reference-only | `docs/features/human-benefit-report.*`, `docs/reference/reference-file-comparison.*`; source sample counts and bounded result remain tied to the source revision. |
| `docs/reference/comprehension-validation-results.md` | reference-only | `docs/features/human-benefit-report.md`, `docs/reference/outcome-report.md`; source limitations are not target verification or release evidence. |

This boundary is intentional: an adopter repository can inherit the target's
documentation route, Contract, evidence and Agent workflow, but must not inherit
another repository's human-subject evidence. A future human-owned study needs
its own consent, retention, privacy and evidence Contract before any claim.

## WI-334 evidence-binding and reuse primitives

WI-334 reads ten pinned source paths individually at
`e5acb677da6621004d96f0ef353c58fe8d3acfbf`. All ten are implemented differently
by design. The Rust target represents content, diff, environment, command,
toolchain, policy, profile, Runtime, stage, and runner identity as one strict
composite `EvidenceContext`; it does not copy the source Python modules or
claim source JSON/API compatibility.

| Pinned source path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/content-bound-evidence-reuse.md` | implemented-different-by-design | `cockpit-evidence` content identity is one component of the composite context; exact bindings only permit advisory reuse. |
| `docs/reference/diff-bound-evidence-reuse.md` | implemented-different-by-design | `DiffIdentity`, repository snapshot facts, and reuse tests bind base/head and changed-path identity; mismatches rerun. |
| `docs/reference/environment-bound-reuse.md` | implemented-different-by-design | Explicit Runtime/toolchain/environment/profile/policy/command/stage fields are bound without serializing the process environment wholesale. |
| `docs/reference/evidence-binding-foundation.md` | implemented-different-by-design | Versioned `ReusableReceipt` validates content-addressed identity, expiry, node, and passed status; it never bypasses protected or required checks. |
| `scripts/ai_evidence_binding.py` | implemented-different-by-design | Typed Rust structs, deny-unknown-fields parsing, and deterministic fail-closed decisions replace the Python builder/validator. |
| `scripts/ai_diff_bound_reuse.py` | implemented-different-by-design | Typed `DiffIdentity` and Git snapshot facts replace the source helper while retaining canonical path/revision mismatch semantics. |
| `scripts/ai_environment_reuse.py` | implemented-different-by-design | Explicit bounded environment inputs and digest fields replace the source adapter; credentials are not read or persisted. |
| `tests/test_ai_evidence_binding.py` | implemented-different-by-design | Rust evidence/repository tests cover strict schema, tampering, mismatch, expiry, failed/protected nodes, and rerun decisions. |
| `tests/test_ai_diff_bound_reuse.py` | implemented-different-by-design | Rust evidence/Git tests cover clean and changed path sets, canonical ordering, malformed paths, policy mismatch, expiry, and immutability. |
| `tests/test_ai_environment_reuse.py` | implemented-different-by-design | Rust evidence/executor tests cover environment and toolchain identity, stale/unknown receipts, protected execution, and digest validation. |

This batch establishes semantic responsibility parity, not source wire parity.
Reuse is an optimization/evidence observation: only an exact fresh binding may
be considered, and the caller still owns governance, coverage, security, and
required-check gates. The inventory, tri-language ledgers, and WI-334 evidence
bind this decision; no source participant, Python, Make, or V1 artifact is
introduced.

## WI-343 inventory foundation reconciliation

WI-339 had already compared the following five pinned paths individually, but
the machine inventory still left them as `deferred-next-batch`. WI-343 registers
those existing decisions deterministically without changing Runtime behavior or
copying source tooling.

| Pinned source path | Classification |
| --- | --- |
| `docs/reference/cross-wi-integration.md` | `reference-only` |
| `docs/reference/dependabot-intake.md` | `not-applicable` |
| `docs/reference/deprecated-assets-registry.json` | `reference-only` |
| `docs/reference/deprecated-assets.md` | `reference-only` |
| `docs/reference/derived-artifacts.md` | `implemented-different-by-design` |

The tri-language ledgers now agree with the generated inventory: 240
implemented-different-by-design, four not-applicable, 30 reference-only, and
582 deferred records, with zero migrate-gap. This is a ledger reconciliation,
not a source command or JSON-wire compatibility claim.

## WI-342 documentation, distribution, and enterprise-boundary batch

WI-342 reads the following ten pinned reference paths individually at
`e5acb677da6621004d96f0ef353c58fe8d3acfbf`. Eight are implemented differently
by design and two are reference-only. The target preserves the reader,
distribution, authority, and enterprise-boundary responsibilities without
copying source-specific Python/Make orchestration, source adopter records, or
provider claims.

| Pinned source path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/distribution.md` | implemented-different-by-design | `docs/release/distribution.*` and the public/N-1 adopter acceptance harness provide immutable Release verification, shared-Runtime installation, repository binding, checksum/SBOM/provenance, and cleanup boundaries. |
| `docs/reference/distribution.ja.md` | implemented-different-by-design | The Japanese route is represented by `docs/release/distribution.ja.md` plus the same target-specific acceptance harness; source Make/Python installer details and bytes are not copied. |
| `docs/reference/documentation-architecture.md` | implemented-different-by-design | `docs/current/README.md`, getting-started/reference routes, tri-language documentation checks, and this ledger preserve canonical layers, reader routes, ownership, and split rules. |
| `docs/reference/documentation-architecture.ja.md` | implemented-different-by-design | Japanese current/getting-started/reference routes preserve the source reader map and language boundary; `.ai/README.md` and explicit Runtime pages remain the instruction boundary. |
| `docs/reference/documentation-authority-boundary.md` | implemented-different-by-design | `.ai/README.md`, `AGENTS.md`, current/reference routes, frontmatter, and documentation acceptance separate current instructions from opt-in reference and historical records. |
| `docs/reference/documentation-authority-registry.json` | implemented-different-by-design | Explicit target routes and metadata checks replace the source topic registry; no global Agent configuration or unverified source topic claim is introduced. |
| `docs/reference/documentation-context-registry.json` | reference-only | Source plan/context labels are source-internal records, not portable Runtime authority or adopter evidence. Target keeps current `.ai` instructions and immutable Work Item/archive history without copying the source registry. |
| `docs/reference/enterprise-control-checklist.md` | implemented-different-by-design | Tri-language enterprise-governance, deployment-boundary, and adopter-configuration pages distinguish repository facts, delegated evidence, retention/audit ownership, and non-certification claims. |
| `docs/reference/enterprise-control-matrix.json` | reference-only | The source observed-control rows are not portable compliance results. Target delegated evidence and policy routes require current external receipts instead of copying `not_verified` source state. |
| `docs/reference/external-identity-boundary.md` | implemented-different-by-design | Typed Rust authority/approval evidence, policy precedence, external evidence import, contract-field documentation, and enterprise pages preserve identity levels without authenticating a person locally. |

The two reference-only records are deliberately not promoted to target
capabilities: source context metadata and source adopter control observations
cannot be transferred as evidence. This is semantic/documentation parity, not
JSON-wire parity. The target's object/adopter boundary remains explicit:
one shared Runtime, repository-scoped `.ai/` state, external provider evidence,
and no organization-wide identity or compliance claim.

The current ledger after this batch is 5,119 records: 4,262
`generated-history`, 240 `implemented-different-by-design`, one
`implemented-equivalent`, four `not-applicable`, 30 `reference-only`, and 582
`deferred-next-batch`; `migrate-gap` remains zero. The 582 deferred records are
still scheduled comparison work and are not parity claims.

## WI-336 first five governance-documentation paths

WI-336 reads the following five paths individually at the pinned reference
commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf`. The results separate
portable governance responsibility from source-specific reports, provider
automation, and historical cleanup tooling.

| Pinned source path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/cross-wi-integration.md` | reference-only | `docs/reference/reference-parity.md`, `docs/reference/outcome-report.md`, and per-Work-Item archive validation provide the target audit boundary. The source's advisory WI-04..WI-13 aggregate report and unobservable conversation receipt are not Runtime commands. |
| `docs/reference/dependabot-intake.md` | not-applicable | Dependabot bot-branch intake is provider-specific. Generic delegated provider evidence and explicit Work Item source binding remain documented in `docs/reference/ci-release-evidence.md` and are not a Dependabot authorization path. |
| `docs/reference/deprecated-assets-registry.json` | reference-only | `.ai/README.md`, `docs/reference/agent-workflow.md`, and exact resource finalization preserve explicit reviewed cleanup and immutable-history boundaries; no source registry or Make scan is shipped. |
| `docs/reference/deprecated-assets.md` | reference-only | The explanatory obsolete-chain and registry hygiene guidance remains source documentation. Rust uses explicit `--repo`, Runtime lifecycle, immutable archives, and resource finalization rather than claiming `check-deprecated-assets`. |
| `docs/reference/derived-artifacts.md` | implemented-different-by-design | `docs/reference/outcome-report.md`, `docs/reference/verification-semantics.md`, `.ai/README.md`, and typed Runtime projections keep Contract/evidence/archive facts separate from status/Outcome views; no source Python registry is required or read as authority. |

This batch is semantic responsibility comparison, not source command or wire
compatibility. Rust does not copy the reference Python scripts, Make targets,
Dependabot workflow, deletion registry, or generated history. The per-Work-Item
archive and human Outcome remain authoritative; derived views cannot authorize a
later decision. The remaining ledger records stay explicitly deferred.

## WI-344 reference documentation batch 14

WI-344 reads the following five pinned reference documents individually. Three
responsibilities are represented by Rust-native reader/runtime boundaries and
two are source-specific historical reports that must not become target
capability or evidence.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/failure-recovery-usability.md` | implemented-different-by-design | `docs/reference/troubleshooting.md`, `docs/features/task-outcome-report.md`, `docs/reference/outcome-report.md`, and typed recovery/Outcome services provide repository-bound failed-gate, recovery-condition, intervention, stop, resolution, and next-action reporting. The source nine-scenario Python report wire shape remains separately staged. |
| `docs/reference/final-north-star-acceptance.json` | implemented-different-by-design | `docs/reference/final-replacement-acceptance.md`, this parity ledger, and the final-replacement harness preserve the twenty dimensions and explicit external-adopter/provider limitations without importing source decision bytes. |
| `docs/reference/final-north-star-acceptance.md` | implemented-different-by-design | Design Philosophy, Product Boundary, Outcome, and final-replacement acceptance preserve the North Star and keep local checks separate from external evidence. |
| `docs/reference/final-wiii-remediation-closure-audit.md` | reference-only | Source WIII PR identities, reviewers, and historical closure claims are not portable target evidence; Rust keeps its own Work Item intelligence and parallelism routes. |
| `docs/reference/full-remediation-acceptance.md` | reference-only | The source WI-01–WI-19 remediation sequence is internal history. The target keeps its own evidence-bound acceptance route and does not publish source progress or release claims. |

This is semantic/documentation parity, not source command or JSON-wire parity.
The companion source recovery and acceptance scripts/tests remain deferred until
their own file-level comparison. The object/adopter boundary remains one shared
Runtime, isolated repository state, and independently bound evidence.

The current ledger is now 5,119 records: 4,262 `generated-history`, 243
`implemented-different-by-design`, one `implemented-equivalent`, four
`not-applicable`, 32 `reference-only`, and 577 `deferred-next-batch`; there are
zero `migrate-gap` records. The deferred count is scheduled work, not a parity
claim.

## WI-345 governance cost and performance documentation batch 15

WI-345 individually compares the next five pinned reference documents at
`e5acb677da6621004d96f0ef353c58fe8d3acfbf`. The two complexity pages remain
reference-only because their Python/Make scanner and source thresholds are not
Rust Runtime behavior. Cost, performance budgets, and profile/cost separation
are represented by Rust-native, repository-bound projections with a narrower
and explicit advisory boundary.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/governance-complexity.ja.md` | reference-only | `docs/reference/governance-complexity.ja.md`, `docs/reference/governance-integrity-gate.ja.md`, and immutable archive rules record the boundary; the source complexity scanner, Make target, and thresholds are not copied. |
| `docs/reference/governance-complexity.md` | reference-only | `docs/reference/governance-complexity.md`, `docs/reference/governance-integrity-gate.md`, and `inspect/status/doctor` preserve repository facts and archive integrity, without claiming source metric equivalence. |
| `docs/reference/governance-cost-metrics.md` | implemented-different-by-design | `ai-cockpit diagnose --repo <repo> [--work-item <id>]`, typed `VerificationCostEstimate`/`VerificationCostObservation`, and `docs/reference/verification-cost.md` provide identity-bound advisory facts; source JSONL phase/wait parsing and wire shape are not Runtime requirements. |
| `docs/reference/governance-performance-budget.md` | implemented-different-by-design | Typed `PerformanceBaseline`/`PerformanceAssessment`, `tests/performance/regression_gate.sh`, and `tests/performance/README.md` enforce explicit local budgets without deriving P95 or weakening required verification. |
| `docs/reference/governance-profile-cost-separation.md` | implemented-different-by-design | `docs/reference/governance-profile-cost-separation.md`, `ci-quality-gates.md`, and `verification-route.md` keep light/standard/strict, operation/stage escalation, VerificationTier, EvidenceAssurance, and cost separate. |

This batch is semantic/documentation parity, not source command or JSON-wire
compatibility. The object/adopter boundary is inherited unchanged: one shared
Runtime, explicit `--repo`, repository-local evidence, policy-owned route
requirements, and advisory cost/performance facts that cannot authorize a
weaker governance result.

The ledger after WI-345 is 5,119 records: 4,262 `generated-history`, 246
`implemented-different-by-design`, one `implemented-equivalent`, four
`not-applicable`, 34 `reference-only`, and 572 `deferred-next-batch`; there are
zero `migrate-gap` records. The 572 deferred records remain scheduled work,
not parity claims.

## WI-346 governance profiles and Cockpit status reading

WI-346 individually compares the six pinned reference documents below at
`e5acb677da6621004d96f0ef353c58fe8d3acfbf`. All six are
`implemented-different-by-design`: the target now gives adopters an explicit
tri-language reading route, while its Rust Runtime, repository context, and
CI boundaries remain different from the source Make/Python orchestration.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/governance-profiles.ja.md` | implemented-different-by-design | `docs/reference/governance-profiles.ja.md`, `governance-profile-cost-separation.ja.md`, `ci-quality-gates.ja.md`, and `verification-route.ja.md` document the same proportional profile intent in Japanese; source dispatch bytes are not copied. |
| `docs/reference/governance-profiles.md` | implemented-different-by-design | `docs/reference/governance-profiles.md`, `governance-profile-cost-separation.md`, `ci-quality-gates.md`, and `verification-route.md` map Light/Standard/Strict, release escalation, mandatory floors, and fail-closed routing to the target's explicit `gate --repo` boundary. |
| `docs/reference/governance-profiles.zh-CN.md` | implemented-different-by-design | The Chinese counterpart pages preserve profile, tier/assurance, cost, and override boundaries without presenting source `make` or Python commands as Rust requirements. |
| `docs/reference/how-to-read-cockpit-status.ja.md` | implemented-different-by-design | `how-to-read-cockpit-status.ja.md`, `outcome-report.ja.md`, and `commands.ja.md` provide a Japanese human handoff route; contract text and source evidence remain authoritative. |
| `docs/reference/how-to-read-cockpit-status.md` | implemented-different-by-design | `how-to-read-cockpit-status.md`, `outcome-report.md`, and `commands.md` map source reader labels to the Rust Outcome sections, colors, stop conditions, and explicit next action. |
| `docs/reference/how-to-read-cockpit-status.zh-CN.md` | implemented-different-by-design | The Chinese counterpart pages provide the same human-safe reading order and evidence boundary; automatic translation cannot alter Contract facts or create approval. |

The six pages deliberately distinguish `VerificationTier` from
`EvidenceAssurance`, and both from advisory cost observations. They explain
that `🟢` is reviewable evidence, `🟡` is incomplete or decision-pending, and
`🔴` is a stop condition; none is merge or release authorization. `unknown`
remains visible and cannot be guessed away. The Rust pages use explicit
`--repo`, preserve the original Contract language, and state the MCP/host
presentation boundary so an adopter can inherit the same behavior.

This is semantic/documentation parity, not source command or JSON-wire parity.
The current ledger after WI-346 is 5,119 records: 4,262
`generated-history`, 252 `implemented-different-by-design`, one
`implemented-equivalent`, four `not-applicable`, 34 `reference-only`, and 566
`deferred-next-batch`; `migrate-gap` remains zero. The 566 deferred records are
still scheduled comparisons, not parity claims.

## WI-347 Knowledge, input trust, installed lifecycle, and capability assessment

WI-347 individually compares the next ten pinned reference paths at
`e5acb677da6621004d96f0ef353c58fe8d3acfbf`. All ten are
`implemented-different-by-design`: the target now publishes reader-facing
Rust-native mappings and explicit limits, while source Python/Make orchestration,
generated assessment bytes, and provider-global behavior remain outside the
Runtime.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/human-report-semantic-quality.md` | implemented-different-by-design | `docs/features/human-benefit-report.md`, `docs/features/task-outcome-report.md`, and `docs/reference/outcome-report.md` preserve the decision-view ordering and forbidden-claim boundary. |
| `docs/reference/implementation-knowledge.ja.md` | implemented-different-by-design | `docs/reference/implementation-knowledge.ja.md` and typed Knowledge records provide the Japanese read-only projection; source filters and generated records are not copied. |
| `docs/reference/implementation-knowledge.md` | implemented-different-by-design | The Rust Knowledge CLI/MCP exposes deterministic repository-bound filters and `KnowledgeV2Record`; broader date/commit/supersession filters remain an explicit non-claim. |
| `docs/reference/implementation-knowledge.zh-CN.md` | implemented-different-by-design | The Chinese Knowledge route documents current filters, evidence binding, and the bounded difference from the source query surface. |
| `docs/reference/input-trust-dataflow.ja.md` | implemented-different-by-design | Japanese provenance guidance maps to typed `FactOrigin`/traceable derivations and fail-closed observation. |
| `docs/reference/input-trust-dataflow.md` | implemented-different-by-design | Typed Rust facts, repository snapshot observation, and input-trust tests preserve source classification and injection boundaries without source JSON wire parity. |
| `docs/reference/input-trust-dataflow.zh-CN.md` | implemented-different-by-design | The Chinese route explains the same provenance, cross-step, and explicit-repository boundary. |
| `docs/reference/installed-lifecycle.md` | implemented-different-by-design | Shared Runtime installation, explicit attach, immutable Release acceptance, and separate migration/rollback boundaries are documented; source installer Python/Make remains reference material. |
| `docs/reference/instruction-traceability.md` | implemented-different-by-design | The inventory, comparison/parity pages, Work Item evidence, and closure receipts provide structural forward/reverse traceability; the source remediation checker is not copied. |
| `docs/reference/japanese-capability-assessment.json` | implemented-different-by-design | Tri-language capability pages and executable presentation/adversarial checks provide bounded coverage; source assessment/corpus bytes and general fluency claims remain reference-bound. |

This is semantic/documentation parity, not source command or JSON-wire parity.
The object/adopter boundary is inherited unchanged: one installed Runtime,
explicit `--repo`, isolated repository facts/evidence, and external provider or
enterprise assurance. Knowledge, provenance, installation, traceability, and
language projections cannot invent authority, benefit, approval, or release
evidence.

The ledger after WI-347 is 5,119 records: 4,262 `generated-history`, 262
`implemented-different-by-design`, one `implemented-equivalent`, four
`not-applicable`, 34 `reference-only`, and 556 `deferred-next-batch` records;
`migrate-gap` remains zero. The 556 deferred records are still scheduled
comparisons, not parity claims.

## WI-348 verification, operation-time policy, and provider-bound batch

WI-348 individually compares the next ten pinned reference paths at
`e5acb677da6621004d96f0ef353c58fe8d3acfbf`. Seven responsibilities are
implemented differently in Rust and three historical provider/pre-release
records are reference-only. The Rust Core adds a strict operation-time
evaluator; it is a policy input and never an executor or provider authority.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/japanese-capability-assessment.md` | implemented-different-by-design | Tri-language Japanese assessment boundary plus localized Outcome, adversarial, installation, and documentation checks; no general fluency claim. |
| `docs/reference/lightweight-verification-and-soft-gates.md` | implemented-different-by-design | Proportional route, content-bound reuse, deterministic partial dependencies, monotonic escalation, and visible advisory boundaries in verification/evidence services. |
| `docs/reference/multilingual-semantic-parity.md` | implemented-different-by-design | Three-language Runtime-owned labels, markers, safety, unknown, decision, limitation, and next-action projections; Contract values remain authoring-language text. |
| `docs/reference/open-pr-issue-reconciliation-662.json` | reference-only | Historical provider inventory; target/provider state requires fresh external observation and cannot authorize a release or merge. |
| `docs/reference/open-pr-issue-reconciliation-662.md` | reference-only | Historical reconciliation narrative; not copied into current status or `.ai/`. |
| `docs/reference/operation-time-policy-reevaluation.ja.md` | implemented-different-by-design | Rust `OperationTimeRequest`/decision evaluator and strict regression tests; source Python trust modules and provider execution are not copied. |
| `docs/reference/operation-time-policy-reevaluation.md` | implemented-different-by-design | Same operation-time boundary with explicit operation, target, scope, authority, freshness, trust, and impact facts. |
| `docs/reference/operation-time-policy-reevaluation.zh-CN.md` | implemented-different-by-design | Chinese reader route for the same fail-closed operation-time evaluator. |
| `docs/reference/performance-diagnosis.md` | implemented-different-by-design | Request-scoped `diagnose` and cost observations report measured execution/reuse facts without inventing provider wait, P95, or assurance. |
| `docs/reference/pre-release-documentation-alignment.json` | reference-only | Historical generated alignment receipt; target documentation uses its own repository-local checks and is never promoted from this source artifact. |

This is semantic parity, not source Python, Make, JSON-wire, or provider-state
parity. The updated ledger contains 5,119 records: 4,262
`generated-history`, 269 `implemented-different-by-design`, one
`implemented-equivalent`, four `not-applicable`, 37 `reference-only`, and 546
`deferred-next-batch`; `migrate-gap` remains zero. The same shared Runtime,
explicit `--repo`, repository-local evidence, and object/adopter isolation apply
to every target project.

## WI-368 — pre-release, adversarial, adopter, and reference-impact batch

WI-368 compares eleven additional pinned paths at
`e5acb677da6621004d96f0ef353c58fe8d3acfbf`, one by one. Six are
`implemented-different-by-design` and five are `reference-only`:

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/pre-release-documentation-alignment.md` | reference-only | Historical generated alignment; current docs use repository-local gates. |
| `docs/reference/pre-release-documentation-review.json` | reference-only | Historical five-strategy review; source findings cannot authorize a target release. |
| `docs/reference/project-test-timing-baseline.json` | implemented-different-by-design | Identity-bound performance samples and advisory budgets; timings never lower verification. |
| `docs/reference/provider-backed-governance-validation.md` | implemented-different-by-design | Provider/hosted controls remain delegated evidence; local checks do not prove them. |
| `docs/reference/real-absurd-injection-cases.{md,zh-CN.md,ja.md}` | implemented-different-by-design | Canonical manifest and Rust tests preserve 15 structured cases and 12 named RAI cases. |
| `docs/reference/real-adopter-reference-validation.md` | implemented-different-by-design | Immutable public Release adopter/upgrade harness with isolated lifecycle and cleanup evidence. |
| `docs/reference/reference-impact-gate.{md,zh-CN.md,ja.md}` | reference-only | The source static scanner/schema/Make surface is not shipped; operation-time policy is a narrower declared-facts boundary. |

The batch also corrects the Standard profile wording so it no longer implies
that a static reference-impact scanner exists. The source adversarial language
pages disagree on named-case count; the target follows the manifest as machine
truth and keeps that discrepancy visible. This is semantic parity and explicit
boundary documentation, not source command or JSON-wire compatibility.

## WI-378 reference documentation batch 17

WI-378 compares the next ten deferred reference paths individually at the
pinned source commit. Nine responsibilities are represented by Rust-native
tri-language documentation and existing Runtime/tests; one generated plan
trace is reference-only. The batch does not copy source Python, Make, provider
configuration, or historical remediation decisions.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/remediation-instruction-traceability.json` | reference-only | `docs/reference/instruction-traceability.md` and the machine inventory explain the current traceability boundary; the source's generated historical plan directives are not target authority. |
| `docs/reference/repository-workflow.ja.md` | implemented-different-by-design | Tri-language `docs/reference/repository-workflow.*`, `.ai/README.md`, and `AGENTS.md` preserve explicit repository context, serial Work Item, reviewed PR, close, and cleanup semantics. |
| `docs/reference/schemas.md` | implemented-different-by-design | Tri-language `schemas.*`, typed Protocol/repository validators, and immutable evidence/decision boundaries map the record families without source wire compatibility. |
| `docs/reference/test-architecture.md` | implemented-different-by-design | Tri-language `test-architecture.*`, CI quality routing, conformance manifest, release/adopter harnesses, and negative-first tests describe layered evidence and external limits. |
| `docs/reference/test-weakening-guard.ja.md` | implemented-different-by-design | Japanese Rust-native weakening route plus snapshot-derived governance signals and regressions; source Python/Make surface is not shipped. |
| `docs/reference/test-weakening-guard.md` | implemented-different-by-design | English Rust-native weakening route, conservative path handling, dynamic profile boundary, and recovery conditions. |
| `docs/reference/test-weakening-guard.zh-CN.md` | implemented-different-by-design | Chinese Rust-native weakening route with fail-closed unknowns, proportional analysis, and explicit non-claims. |
| `docs/reference/troubleshooting.ja.md` | implemented-different-by-design | Japanese stop-state/recovery route, command reference, installed-lifecycle boundary, and documentation checks replace source wizard/Make instructions. |
| `docs/reference/troubleshooting.md` | implemented-different-by-design | English stop-state/recovery route with explicit toolchain, adopter, active-Work-Item, and evidence-preservation boundaries. |
| `docs/reference/upgrade.ja.md` | implemented-different-by-design | Japanese Runtime-upgrade versus repository-migration route with immutable Release, rollback, and history-preservation rules. |

The updated ledger remains 5,119 records: 4,262 `generated-history`, 284
`implemented-different-by-design`, one `implemented-equivalent`, four
`not-applicable`, 43 `reference-only`, and 525 `deferred-next-batch`; there are
zero `migrate-gap` records. The deferred set remains scheduled comparison work,
not a parity claim.

## WI-379 reference documentation batch 18

WI-379 compares the next ten deferred reference paths individually at the
pinned source commit. Eight responsibilities are represented by Rust-native
tri-language documentation and two historical audit files remain
`reference-only`. The batch adds no Runtime code and does not copy source
Python, Make, provider configuration, or generated history.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/upgrade.md` | implemented-different-by-design | Tri-language `upgrade.*`, `installed-lifecycle.*`, and explicit migration/conflict/rollback boundaries; source installer commands are explanatory only. |
| `docs/reference/verification-evidence-reuse-runtime.md` | implemented-different-by-design | `verification-evidence-reuse-runtime.*`, `verification-route.*`, `verification-semantics.*`, typed identity-bound receipts, protected-node execution, and observable reuse metrics. |
| `docs/reference/verification-evidence-reuse.md` | implemented-different-by-design | `verification-evidence-reuse.*`, `verification-cost.*`, and `verification-planner.*`; exact binding/invalidation and advisory call-count boundary. |
| `docs/reference/verification-fixture-boundary.md` | implemented-different-by-design | `verification-fixture-boundary.*` and repository-native tests; local fixtures exclude runtime/cache state and cannot prove provider/adopter evidence. |
| `docs/reference/wi01-wi20-bidirectional-traceability-audit.json` | reference-only | Historical generated V1 audit bytes; current target truth is the pinned inventory, Work Item archive, evidence, and tri-language traceability pages. |
| `docs/reference/wi01-wi20-bidirectional-traceability-audit.md` | reference-only | Historical narrative bound to source Python/Make evidence; it is not copied or treated as current target authority. |
| `docs/reference/wiii-v2-integration-audit.md` | implemented-different-by-design | `wiii-v2-integration-audit.*`, Rust `status`/intelligence projection, explicit schema/source identity checks, and no scheduler/provider claims. |
| `docs/reference/work-item-intelligence-performance-baseline.md` | implemented-different-by-design | `work-item-intelligence-performance-baseline.*`, `diagnose`, and advisory cost/performance observations; source benchmark numbers are not claimed. |
| `docs/reference/work-item-lifecycle-closure.ja.md` | implemented-different-by-design | `work-item-lifecycle-closure.*`, `repository-workflow.*`, and Runtime `finalize`/`close` receipts with exact PR/base/branch/worktree cleanup. |
| `docs/reference/work-item-lifecycle-closure.md` | implemented-different-by-design | Same Rust-native closure route and recovery boundary in English; source `make`/Python orchestration is not a command requirement. |

This is semantic/documentation parity, not source command, JSON-wire, or
provider-state compatibility. The object/adopter boundary remains one shared
Runtime with explicit `--repo` and isolated repository facts, Work Items,
evidence, knowledge, and snapshots. The ledger after WI-379 contains 4,262
`generated-history`, 292 `implemented-different-by-design`, one
`implemented-equivalent`, four `not-applicable`, 45 `reference-only`, and 515
`deferred-next-batch` records; `migrate-gap` remains zero.

## WI-386 — reference documentation batch 19

WI-386 compares four deferred reference documents one by one at pinned source
commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf`. Two historical/internal
documents remain `reference-only`; the roadmap and security-boundary
responsibilities are represented by current Rust-native documentation. This
batch does not copy source Python, Make commands, provider configuration,
historical GO/NO-GO claims, or future roadmap milestones as shipped features.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/review-final-evidence.md` | reference-only | Generated R11 evidence index bound to source-specific `make` checks and historical review state. Current `final-replacement-acceptance.md`, `ci-release-evidence.md`, and repository-local Work Item evidence generate fresh, identity-bound truth; no historical GO/NO-GO is copied. |
| `docs/review-remediation-backlog.md` | reference-only | Internal R0–R11 backlog and Python/Make execution plan. Current `repository-workflow.md`, `governance-integrity-gate.md`, and this comparison ledger are the maintained boundaries; the source plan is not current authority. |
| `docs/roadmap.md` | implemented-different-by-design | `docs/philosophy.md`, `docs/architecture.md`, and `docs/capabilities.md` preserve mission, evidence governance, intent, human control, repository intelligence, and organization-policy direction. Historical V1–V4 milestones and source roadmap wording are not shipped capability claims. |
| `docs/security-boundaries.md` | implemented-different-by-design | `docs/security/threat-model.md`, `docs/reference/input-trust-dataflow.md`, `docs/reference/operation-time-policy-reevaluation.md`, and `docs/security/adversarial-validation.md` preserve content/authority separation, deterministic fail-closed handling, high-risk re-evaluation, and limitations. The source classifier implementation is not copied. |

This is semantic/documentation parity, not source command, JSON-wire, or
provider-state compatibility. Every object/adopter project inherits the
Rust-native documentation boundary from the shared Runtime, while repository
facts, Work Items, evidence, knowledge, and snapshots remain isolated behind
explicit `--repo`. The ledger after WI-386 contains 4,262
`generated-history`, 294 `implemented-different-by-design`, one
`implemented-equivalent`, four `not-applicable`, 47 `reference-only`, and 511
`deferred-next-batch` records; `migrate-gap` remains zero.

## WI-387 — reference documentation batch 20

WI-387 compares the next four deferred security and supply-chain documents one
by one at pinned source commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf`.
Their responsibilities are represented by Rust-native security, trust-flow,
release-evidence, and distribution documentation. This batch preserves the
bounded repository-governance response and delegated external-control boundary;
it does not claim to ship a general prompt-injection detector or generate
signatures, SBOM, provenance, or provider assurance.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/security/injection-boundary.ja.md` | implemented-different-by-design | `docs/security/adversarial-validation.ja.md`, `docs/reference/input-trust-dataflow.ja.md`, and `docs/reference/operation-time-policy-reevaluation.ja.md` preserve the Japanese bounded injection response, fail-closed operation-time review, and explicit external-control limits. |
| `docs/security/injection-boundary.md` | implemented-different-by-design | `docs/security/adversarial-validation.md`, `docs/reference/input-trust-dataflow.md`, and `docs/reference/operation-time-policy-reevaluation.md` preserve the bounded repository-governance response; untrusted text remains data and the source page is not copied as a general detector claim. |
| `docs/security/injection-boundary.zh-CN.md` | implemented-different-by-design | `docs/security/adversarial-validation.zh-CN.md`, `docs/reference/input-trust-dataflow.zh-CN.md`, and `docs/reference/operation-time-policy-reevaluation.zh-CN.md` preserve the Chinese boundary, deterministic fail-closed handling, and non-claims. |
| `docs/security/supply-chain.md` | implemented-different-by-design | `docs/security/threat-model.md`, `docs/reference/ci-release-evidence.md`, `docs/release/distribution.md`, and `docs/getting-started/security-release-verification.md` preserve delegated supply-chain evidence ownership and exact artifact binding; external trust roots remain outside the Runtime. |

The ledger after WI-387 contains 4,262 `generated-history`, 298
`implemented-different-by-design`, one `implemented-equivalent`, four
`not-applicable`, 47 `reference-only`, and 507 `deferred-next-batch` records;
`migrate-gap` remains zero. The same Rust-native security and supply-chain
boundaries are inherited by every attached object/adopter repository, while
repository facts and evidence remain isolated by explicit `--repo` context.

## WI-388 — reference documentation batch 21

WI-388 compares six deferred reference documents one by one at pinned source
commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf`. Their responsibilities are
represented by Rust-native threat-model, adoption, release-evidence,
installation, and troubleshooting routes. This batch records the distributed
counterparts and their evidence boundaries without copying source commands or
historical stability claims.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/security/threat-model.md` | implemented-different-by-design | `docs/security/threat-model.md`, `.zh-CN.md`, and `.ja.md` preserve protected assets, trust boundaries, fail-closed threats, and explicit external-control limits; no claim is made to detect every malicious intention or certify enterprise security. |
| `docs/template-adopter-stability-matrix.md` | implemented-different-by-design | `docs/reference/final-replacement-acceptance.md`, `docs/getting-started/standard-adoption-guide.md`, `docs/reference/ci-release-evidence.md`, and `tests/release/adopter_acceptance.sh` distribute template, adoption, lifecycle, and evidence-kind boundaries; template-only runs are not promoted to external stability proof. |
| `docs/troubleshooting.md` | implemented-different-by-design | The tri-language `docs/reference/troubleshooting.*` route provides stop states, recovery, evidence preservation, and explicit repository-bound commands rather than a compatibility-only redirect. |
| `docs/troubleshooting/installation.ja.md` | implemented-different-by-design | `docs/getting-started/installation.ja.md`, `installation-security.ja.md`, and `docs/reference/troubleshooting.ja.md` preserve uncertainty stops, strict Release verification, and explicit attachment without source wizard commands. |
| `docs/troubleshooting/installation.md` | implemented-different-by-design | `docs/getting-started/installation.md`, `installation-security.md`, and `docs/reference/troubleshooting.md` preserve uncertainty stops, strict Release verification, and explicit attachment without silently selecting moving or older artifacts. |
| `docs/troubleshooting/installation.zh-CN.md` | implemented-different-by-design | `docs/getting-started/installation.zh-CN.md`, `installation-security.zh-CN.md`, and `docs/reference/troubleshooting.zh-CN.md` preserve the Chinese recovery route, strict artifact binding, and explicit repository context. |

This is semantic/documentation parity, not source command, JSON-wire, or
provider-state compatibility. Every attached object/adopter repository
inherits the same Rust-native threat, adoption, installation, and recovery
boundaries from the shared Runtime, while repository facts and evidence remain
isolated by explicit `--repo`. The ledger after WI-388 contains 4,262
`generated-history`, 304 `implemented-different-by-design`, one
`implemented-equivalent`, four `not-applicable`, 47 `reference-only`, and 501
`deferred-next-batch` records; `migrate-gap` remains zero.

## WI-389 — reference documentation batch 22

WI-389 compares six deferred reference documents one by one at pinned source
commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf`. The uninstall guidance is
represented by the installed-lifecycle route, and upgrade guidance by the
Rust-native upgrade reference. The batch preserves proposal-before-write,
owner confirmation, immutable Release binding, rollback, conflict stops, and
explicit active-recovery boundaries without copying source installer commands.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/troubleshooting/uninstall.ja.md` | implemented-different-by-design | `docs/reference/installed-lifecycle.ja.md` preserves read-only inventory, owner confirmation, proposal and separate execution confirmation, bounded removal, receipt verification, evidence retention, and fail-closed unknown recovery. |
| `docs/troubleshooting/uninstall.md` | implemented-different-by-design | `docs/reference/installed-lifecycle.md` preserves read-only inventory, owner confirmation, proposal and separate execution confirmation, bounded removal, receipt verification, evidence retention, and fail-closed unknown recovery. |
| `docs/troubleshooting/uninstall.zh-CN.md` | implemented-different-by-design | `docs/reference/installed-lifecycle.zh-CN.md` preserves read-only inventory, owner confirmation, proposal and separate execution confirmation, bounded removal, receipt verification, evidence retention, and fail-closed unknown recovery. |
| `docs/upgrade.ja.md` | implemented-different-by-design | `docs/reference/upgrade.ja.md` preserves immutable Release/runtime identity, rollback-safe active configuration, conflict and downgrade stops, explicit migration, and separately reviewed `--upgrade-with-active` recovery. |
| `docs/upgrade.md` | implemented-different-by-design | `docs/reference/upgrade.md` preserves immutable Release/runtime identity, rollback-safe active configuration, conflict and downgrade stops, explicit migration, and separately reviewed `--upgrade-with-active` recovery. |
| `docs/upgrade.zh-CN.md` | implemented-different-by-design | `docs/reference/upgrade.zh-CN.md` preserves immutable Release/runtime identity, rollback-safe active configuration, conflict and downgrade stops, explicit migration, and separately reviewed `--upgrade-with-active` recovery. |

This is semantic/documentation parity, not source command, JSON-wire, or
provider-state compatibility. Every attached object/adopter repository
inherits the same Rust-native uninstall, upgrade, rollback, and recovery
boundaries from the shared Runtime, while repository facts and evidence remain
isolated by explicit `--repo`. The ledger after WI-389 contains 4,262
`generated-history`, 310 `implemented-different-by-design`, one
`implemented-equivalent`, four `not-applicable`, 47 `reference-only`, and 495
`deferred-next-batch` records; `migrate-gap` remains zero.

## WI-390 — reference Work Item style guide

WI-390 compares the pinned `docs/work-item-style-guide.md` one section at a
time. Its reader-facing guidance is represented by the tri-language Rust-native
style guide and linked Contract/workflow references. The target keeps the
source's outcome-first writing, explicit problem and boundaries, observable
acceptance, human-owned decisions, minimal sufficient process, executable
verification, and documentation-before-schema principles.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/work-item-style-guide.md` | implemented-different-by-design | `docs/reference/work-item-style-guide.md`, `.zh-CN.md`, and `.ja.md`, linked from the reference index and grounded in `contract-fields` and `repository-workflow`. The page preserves human-owned intent/problem/constraints/rationale, explicit scope/non-goals, machine-checkable acceptance, executable verification, proportional profiles, and per-repository object/adopter inheritance. It does not copy reference metadata, Python/Make commands, installer behavior, or Runtime implementation. |

This is semantic/documentation parity, not source command or JSON-wire
compatibility. The shared Runtime remains external to every adopter project;
each attached repository inherits the same reader-facing boundary through its
own `.ai/` and adapter, while Contract, evidence, knowledge, and repository
identity stay isolated by explicit `--repo`. The ledger after WI-390 contains
4,262 `generated-history`, 311 `implemented-different-by-design`, one
`implemented-equivalent`, four `not-applicable`, 47 `reference-only`, and 494
`deferred-next-batch` records; `migrate-gap` remains zero.

## WI-391 — C# adaptation example

WI-391 compares the pinned `examples/csharp/README.md` section by section at
source commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf`. The source's four
concerns—installation, .NET quality checks and coverage boundaries, Contract
design, and guideline-compliance evidence—are represented by a tri-language
Rust-native C# adaptation page and existing installation, Contract, and
verification references.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `examples/csharp/README.md` | implemented-different-by-design | `docs/reference/csharp-adaptation.md`, `.zh-CN.md`, and `.ja.md`, with links to the shared Runtime installation, Contract fields, and verification route. The source semantics are retained, but `install.sh`, `Makefile.ai.stack`, source guard/Python orchestration, and legacy JSON-wire examples remain external or non-compatible by design. |

This is semantic/documentation parity, not C# toolchain support or a second
technology adopter acceptance claim. A future C# adopter receipt must use an
immutable public Release and its own repository context. The shared Runtime is
installed once outside the adopter, while `.ai/`, Contract, evidence, and
project policy remain repository-local and isolated by explicit `--repo`.
The ledger after WI-391 contains 4,262 `generated-history`, 312
`implemented-different-by-design`, one `implemented-equivalent`, four
`not-applicable`, 47 `reference-only`, and 493 `deferred-next-batch` records;
`migrate-gap` remains zero.

## WI-392 — Android fixture adaptation

WI-392 compares the four pinned Android fixture files one by one at source
commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf`. Kotlin source and test
semantics are mapped to adopter-owned paths and commands; fixture metadata and
Gradle topology are mapped to Project Profile/Observer facts with explicit
unknown boundaries.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `examples/fixtures/android-app/app/src/main/kotlin/example/MainActivity.kt` | implemented-different-by-design | `docs/reference/android-fixture-adaptation.md` maps the source path to explicit Contract scope and keeps Kotlin execution provider-owned. |
| `examples/fixtures/android-app/app/src/test/kotlin/example/MainActivityTest.kt` | implemented-different-by-design | The adaptation guide maps the `kotlin.test` assertion to an owner-confirmed Gradle verification command; a test file does not prove SDK/device/CI readiness. |
| `examples/fixtures/android-app/fixture.json` | implemented-different-by-design | Project Profile/Observer may record stack/toolchain/platform/path facts; `installerStack` is not a Runtime install contract and platform labels are not evidence. |
| `examples/fixtures/android-app/settings.gradle.kts` | implemented-different-by-design | Gradle repository/module topology is recorded as bounded context; dependency, SDK, credential, network, and hosted-CI readiness remain Unknown until evidence exists. |

This is semantic/documentation parity, not Android toolchain support, build
execution, or source JSON-wire compatibility. Installation intentionally uses
one shared immutable Runtime outside each adopter plus explicit `attach --repo`;
the reference fixture's Gradle files, SDK installation, and installer behavior
are not copied. The ledger after WI-392 contains 4,262 `generated-history`,
316 `implemented-different-by-design`, one `implemented-equivalent`, four
`not-applicable`, 47 `reference-only`, and 489 `deferred-next-batch` records;
`migrate-gap` remains zero.

## WI-393 — Flutter fixture adaptation

WI-393 compares the four pinned Flutter fixture files one by one at source
commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf`. Dart source and test
semantics are mapped to adopter-owned paths and commands; fixture and package
metadata are mapped to Project Profile/Observer facts with explicit unknown
boundaries.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `examples/fixtures/flutter-app/fixture.json` | implemented-different-by-design | `docs/reference/flutter-fixture-adaptation.md` maps project type, stack, toolchain, platforms, and safe/test paths to bounded Profile/Contract facts. `installerStack` is not a Runtime installation contract. |
| `examples/fixtures/flutter-app/lib/main.dart` | implemented-different-by-design | The `greeting()` source path is adopter-owned Contract scope; Dart execution remains owner/provider-owned and is not inferred by the Runtime. |
| `examples/fixtures/flutter-app/pubspec.yaml` | implemented-different-by-design | Package name and Dart SDK range are observable metadata; SDK, dependency, network, and lockfile readiness remain Unknown until evidence exists. |
| `examples/fixtures/flutter-app/test/widget_test.dart` | implemented-different-by-design | The `flutter_test` assertion maps to an owner-confirmed provider command; the file alone does not prove SDK, platform runner, plugin, or hosted-CI readiness. |

This is semantic/documentation parity, not Flutter toolchain support, build
execution, or source JSON-wire compatibility. Installation intentionally uses
one shared immutable Runtime outside each adopter plus explicit `attach --repo`;
Flutter SDK/package installation and the reference install implementation are
not copied. The ledger after WI-393 contains 4,262 `generated-history`, 320
`implemented-different-by-design`, one `implemented-equivalent`, four
`not-applicable`, 47 `reference-only`, and 485 `deferred-next-batch` records;
`migrate-gap` remains zero.

## WI-394 — iOS Swift Package fixture adaptation

WI-394 compares the four pinned iOS Swift Package fixture files one by one at
source commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf`. Swift package,
source, and XCTest semantics are mapped to adopter-owned paths and commands;
fixture metadata is mapped to Project Profile/Observer facts with explicit
unknown boundaries.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `examples/fixtures/ios-swift-package/Package.swift` | implemented-different-by-design | The SwiftPM product/target topology is adopter/provider-owned build metadata; SDK/Xcode readiness is not inferred by the Runtime. |
| `examples/fixtures/ios-swift-package/Sources/AppCore/AppCore.swift` | implemented-different-by-design | The `greeting()` source path is adopter-owned Contract scope; Swift execution remains provider-owned. |
| `examples/fixtures/ios-swift-package/Tests/AppCoreTests/AppCoreTests.swift` | implemented-different-by-design | The XCTest assertion maps to an owner-confirmed `swift test` or Xcode command; the file alone does not prove SDK, simulator, signing, or hosted-CI readiness. |
| `examples/fixtures/ios-swift-package/fixture.json` | implemented-different-by-design | Project Profile/Observer may record package/toolchain/platform/path facts; `installerStack` and `macos` are metadata, not shared Runtime installation or execution evidence. |

This is semantic/documentation parity, not Apple toolchain support, build
execution, or source JSON-wire compatibility. Installation intentionally uses
one shared immutable Runtime outside each adopter plus explicit `attach --repo`;
SwiftPM/Xcode installation, SDK selection, and source installer behavior are
not copied. The ledger after WI-394 contains 4,262 `generated-history`, 324
`implemented-different-by-design`, one `implemented-equivalent`, four
`not-applicable`, 47 `reference-only`, and 481 `deferred-next-batch` records;
`migrate-gap` remains zero.

## WI-421 — mixed-monorepo fixture boundary

WI-421 compares the five pinned files under
`examples/fixtures/mixed-monorepo/` one by one at source commit
`e5acb677da6621004d96f0ef353c58fe8d3acfbf`. They are executable application
fixtures, not Rust Runtime code or portable enterprise evidence. Each path is
therefore recorded as `reference-only` with an explicit adopter boundary.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `fixture.json` | reference-only | Records the sample's mixed Python/Node markers, platforms, and safe/test paths. Project Observer/Profile may record observed facts, but the Runtime does not infer toolchain capability or safe scope from this file. |
| `package.json` | reference-only | Package metadata is fixture application input. Node installation, dependencies, scripts, and execution remain adopter/provider responsibilities. |
| `pyproject.toml` | reference-only | Python packaging metadata is not a portable Contract or Runtime dependency. Python installation, dependencies, and test commands require explicit adopter evidence. |
| `services/api/app.py` | reference-only | The health function is application code, not governance logic. The Runtime can bind an adopter-declared argv result but does not ship or infer Python behavior. |
| `services/api/tests/test_app.py` | reference-only | The pytest assertion is fixture evidence only. An adopter must declare and run its own verification command; source tests are never promoted as target evidence. |

This comparison preserves the useful governance meaning—observed facts,
explicit scope, provider-owned execution, and evidence binding—without copying
the mixed fixture, Python/Node toolchains, installer behavior, or source JSON
wire shape. Every attached object/adopter project inherits the same shared
Runtime Contract, lifecycle, evidence, knowledge, and human Outcome controls;
its repository identity and facts remain isolated under explicit `--repo`.
This is not mixed-stack toolchain support or a second-technology adopter
acceptance. The ledger after WI-421 contains 4,262 `generated-history`, 324
`implemented-different-by-design`, one `implemented-equivalent`, four
`not-applicable`, 65 `reference-only`, and 463 `deferred-next-batch` records;
`migrate-gap` remains zero.
