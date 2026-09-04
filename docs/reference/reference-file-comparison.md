---
author: AI Cockpit maintainers
title: "Reference File Comparison"
description: "The pinned, staged method for comparing the reference source file by file."
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-563-reference-file-comparison-batch-43
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
- Rust comparison baseline: [`xinglun/ai-cockpit`](https://github.com/xinglun/ai-cockpit) `origin/main` at `89c9e63b1733ad77a58d1544105bde8ba24cf877`.
- Runtime used for the comparison work: the published `ai-cockpit 0.2.72` binary, SHA256 `sha256:405247cc11f30664ab6337fd36f47a96a9d6c4907f3821077987d7fc365a85dd`.

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
The ledger's `targetCommit` is its historical rebaseline anchor; the current
Runtime baseline above is the reviewed `origin/main` tip used for this batch.

## Safe ledger commands

`python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --check --source-commit fde3380f81fea5fd2e288f7a8849f737dc074060 --target-commit cb8248fdf8ac8d965d8d8eb7b53760147bd13fcd` is read-only. `--check` rejects `--reference`, `--target`, `--rebaseline-from`, and every `--apply-*` option before loading or writing a manifest; use those options only for an explicit generation or update operation. The conformance wrapper proves the rejection and byte identity of the checked ledger.

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

## WI-512 — governance reference pages and verification boundaries

WI-512 re-read the following twelve source paths one by one at the pinned local
reference commit. Two paths were already classified by WI-504 and remain
historically linked; the other ten receive the current batch classification.
The target's translated pages are listed as counterparts even when the source
checkout has no translation file. Every row is semantic parity, not source
command, Python-module, or JSON-wire compatibility.

| Pinned reference path | Classification | Rust counterpart and non-claim |
| --- | --- | --- |
| `docs/reference/schemas.md` | implemented-different-by-design | `docs/reference/schemas.*`, typed Protocol and schema tests; source YAML registries and Python validators are not copied. |
| `docs/reference/test-architecture.md` | implemented-different-by-design | `docs/reference/test-architecture.*`, CI quality route and governance gates; VerificationTier remains separate from EvidenceAssurance. |
| `docs/reference/test-weakening-guard.md` | implemented-different-by-design | tri-language weakening pages, Rust governance signals and regressions; source Make/Python implementation is not a Runtime dependency. |
| `docs/reference/test-weakening-guard.zh-CN.md` | implemented-different-by-design | Chinese presentation counterpart and the same typed Rust guard boundary; locale bytes do not grant policy authority. |
| `docs/reference/test-weakening-guard.ja.md` | implemented-different-by-design | Japanese presentation counterpart and the same typed Rust guard boundary; locale bytes do not grant policy authority. |
| `docs/reference/verification-fixture-boundary.md` | implemented-different-by-design | tri-language fixture boundary plus adopter/isolation manifests; source fixture helper bytes are not copied. |
| `docs/reference/troubleshooting.md` | implemented-different-by-design (WI-504, revalidated) | explicit `--repo` Runtime recovery and tri-language troubleshooting; provider wizard/toolchain commands remain external. |
| `docs/reference/troubleshooting.ja.md` | implemented-different-by-design | Japanese recovery route and adapter boundary; source wizard/session implementation is not copied. |
| `docs/reference/upgrade.md` | implemented-different-by-design | tri-language upgrade, distribution and migration docs; shared Runtime upgrade does not rewrite repository evidence. |
| `docs/reference/upgrade.ja.md` | implemented-different-by-design | Japanese Runtime/migration boundary; source installer, provider markers and locale JSON are not copied. |
| `docs/reference/work-item-lifecycle-closure.md` | implemented-different-by-design (WI-504, revalidated) | tri-language closure, finalize/recovery and ready-on-base checks; source Make/Python recovery orchestration is not a Rust command. |
| `docs/reference/work-item-lifecycle-closure.ja.md` | implemented-different-by-design | Japanese closure and historical recovery boundary; provider-specific routes remain external. |

The target and every adopter inherit the shared external Runtime, isolated
repository context, Contract/evidence/knowledge records, and human Outcome
boundary. They do not inherit source-specific installers, Make targets,
provider decisions, or generated history. The current ledger contains 4,262
`generated-history`, 340 `implemented-different-by-design`, 1
`implemented-equivalent`, 4 `not-applicable`, 90 `reference-only`, and 439
`deferred-next-batch` records; `migrate-gap` remains zero.

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

<!-- reference-inventory-counts: total=4450 generated-history=3681 implemented-different-by-design=403 implemented-equivalent=1 not-applicable=7 reference-only=104 deferred-next-batch=254 migrate-gap=0 -->

The machine-checked table below is the single source for the current snapshot;
the same canonical keys are used in all three language pages. The current
reference set has 4,450 paths. The append-only ledger has 5,119 records because
it retains 669 retired paths from the previous reference baseline. Deferred
records remain scheduled work, not parity claims. The rebaseline records 160
changed current paths, and the capability/profile slice has no remaining
`migrate-gap` records:

| Metric | Count |
| --- | ---: |
| `current-tracked-paths` | 4,450 |
| `generated-history` | 3,681 |
| `implemented-different-by-design` | 403 |
| `implemented-equivalent` | 1 |
| `not-applicable` | 7 |
| `reference-only` | 104 |
| `deferred-next-batch` | 254 |
| `migrate-gap` | 0 |
| `retired-reference-paths` | 669 |
| `append-only-ledger-records` | 5,119 |

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

## WI-441 local-reference entrypoint and Agent parity

WI-441 re-reads nine entrypoint and Agent-facing files at the maintained local
reference commit `fde3380f81fea5fd2e288f7a8849f737dc074060`. The reference
checkout is `/Users/sei-rinn/dev/workspace_python/ai-cockpit-template`; the
public repository is not accessed and hosted CI uses only the committed offline
corpus.

| Pinned local reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `AGENTS.md` | implemented-different-by-design | `AGENTS.md`, `.ai/README.md`, `docs/reference/agent-workflow.md`, and typed lifecycle/adapter services preserve Contract-first work, latest-base discovery, human pause, closure, and cleanup. Source `make ai-*` commands remain source-only. |
| `GEMINI.md` | implemented-different-by-design | The explicit Gemini adapter generated by `crates/cockpit-agent` projects the portable Contract/Summary/checkpoint guidance; a provider-specific root file and global configuration are not copied. |
| `docs/README.md` | implemented-different-by-design | The target's current/getting-started/operations/reference map retains the source reader-first and goal-first intent with Rust-specific boundaries. |
| `docs/README.zh-CN.md` | implemented-different-by-design | The Chinese reader route preserves the same intent and language links while making Runtime/adopter ownership explicit. |
| `docs/README.ja.md` | implemented-different-by-design | The Japanese reader route preserves the same intent and language links while making Runtime/adopter ownership explicit. |
| `docs/capabilities.md` | implemented-different-by-design | The target keeps the Repository Governance Layer and external non-claims, with concrete Rust CLI/MCP, scaffold, profile, knowledge, Outcome, and isolation paths. |
| `docs/capabilities.zh-CN.md` | implemented-different-by-design | The Chinese capability route keeps the source boundary and documents repository-local Runtime/adopter inheritance without copying source status bytes. |
| `docs/capabilities.ja.md` | implemented-different-by-design | The Japanese capability route keeps the source boundary and documents repository-local Runtime/adopter inheritance without copying source status bytes. |
| `docs/features/task-outcome-report.md` | implemented-different-by-design | `OutcomeV2`, CLI/MCP human handoff, and immutable evidence retain report/status/PR separation; source prose and Make commands are not wire requirements. |

All nine records are now individually resolved. This is semantic parity, not
source file or JSON-wire parity: the Rust repository does not need a committed
`GEMINI.md` because `agent install --provider gemini` is explicit, owned,
reversible, and repository-bound. Adopter repositories inherit the same shared
Runtime and isolated `.ai/` boundary.

## WI-461 — getting-started onboarding rebaseline

WI-461 re-reads the nine onboarding pages changed between the historical
comparison commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` and the maintained
local reference commit `fde3380f81fea5fd2e288f7a8849f737dc074060`. The source
checkout is `/Users/sei-rinn/dev/workspace_python/ai-cockpit-template`; no
public reference or source implementation is copied.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/getting-started/first-work-item.md` | implemented-different-by-design | Rust tri-language first-Work-Item pages preserve the complete repository-bound lifecycle, visible Outcome, review stop, and exact cleanup with native CLI commands; the source Make command and removed `REPORT_LANGUAGE` argument are not copied. |
| `docs/getting-started/first-work-item.zh-CN.md` | implemented-different-by-design | The Chinese page preserves the same lifecycle and explicit `--repo` boundary; presentation language does not alter Contract facts. |
| `docs/getting-started/first-work-item.ja.md` | implemented-different-by-design | The Japanese page preserves the same lifecycle and provider-resource boundary; its duplicated merge paragraph was corrected in this batch. |
| `docs/getting-started/security-release-verification.md` | implemented-different-by-design | Rust release/distribution and installation-security pages preserve tag, digest, SBOM, provenance, provider-responsibility, and adopter-isolation semantics through the current manifest/SHA256SUMS route; source `release.json` projection is not copied. |
| `docs/getting-started/security-release-verification.zh-CN.md` | implemented-different-by-design | The Chinese release route keeps evidence separation and fail-closed mismatch handling with Rust-native assets and external-provider boundaries. |
| `docs/getting-started/security-release-verification.ja.md` | implemented-different-by-design | The Japanese release route keeps digest, provenance, SBOM, and public-adopter limits without importing source installer behavior. |
| `docs/getting-started/standard-adoption-guide.md` | implemented-different-by-design | The Rust guide retains reader-first install, attach, calibration, adapter, Work Item, Outcome, merge, cleanup, and close stages with the shared repository-bound Runtime. |
| `docs/getting-started/standard-adoption-guide.zh-CN.md` | implemented-different-by-design | The Chinese guide preserves ordered adoption boundaries and explicit repository ownership with Rust CLI routes. |
| `docs/getting-started/standard-adoption-guide.ja.md` | implemented-different-by-design | The Japanese guide preserves the ordered adoption route and shared Runtime boundary without source-specific commands. |

All nine records are now individually resolved. This is semantic/documentation
parity, not source-file or JSON-wire parity. The inventory retains
`sourceChangedSincePrevious`, `previousBatch`, and `previousClassification` as
comparison provenance while removing the deferred status for this batch.

## WI-464 — workflow and build rebaseline

WI-464 re-reads the four source paths whose bytes changed after the earlier
workflow comparisons at the maintained local reference commit
`fde3380f81fea5fd2e288f7a8849f737dc074060`. No source implementation is copied.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `.github/workflows/compatibility.yml` | implemented-different-by-design | Source ShellCheck installation and Python/multi-stack matrix remain source/provider concerns. Rust keeps its reviewed action-runtime policy, dynamic quality route, Rust workspace/platform gates, and public adopter acceptance. |
| `.github/workflows/release.yml` | implemented-different-by-design | Source `release-digests.json` archive projection and removal of the obsolete `release.json` dual-asset check map to Rust release-manifest/`SHA256SUMS`, SBOM/provenance, platform smoke, and adopter evidence. Source projection bytes are not copied. |
| `.github/workflows/smoke.yml` | implemented-different-by-design | Source removes a `REPORT_LANGUAGE` Make argument. Rust has no source `smoke.yml`; CI, release, gate-manifest, and immutable adopter harnesses provide the bounded checks with explicit repository context. |
| `Makefile` | implemented-different-by-design | Source Python/Make shard, knowledge, and language helpers remain source-only. Rust uses Cargo, the CLI, the canonical gate manifest, and explicit `--repo`; no second Make governance layer is required. |

The target action pins remain governed by the target's own reviewed action-runtime
policy; the source matrix pin is not silently substituted. The ledger resolves
these four source-change records while retaining their
`sourceChangedSincePrevious` provenance. No Rust omission was found, and object
or adopter repositories inherit the shared Runtime and isolated repository
evidence boundary rather than source workflow files. This is semantic/documentation
parity, not source-file, provider, Python/Make, or JSON-wire compatibility.

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

## WI-475 — Outcome, event, and quality-gate reference comparison

WI-475 re-reads seven paths changed in the maintained local reference at
`fde3380f81fea5fd2e288f7a8849f737dc074060`. The comparison is section by
section and records a bounded semantic decision; source Python/Make/provider
bytes are not copied into the Rust repository.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/features/human-benefit-report.md` | implemented-different-by-design | Rust `OutcomeV2`/`humanHandoff`, Task Outcome references, and CLI/MCP tests preserve deterministic human projection, evidence-count semantics, archive ownership, and explicit non-claims. Source `ai-finish`/`check-ai-pr` report files remain source/provider surfaces. |
| `docs/features/human-benefit-report.zh-CN.md` | implemented-different-by-design | The Chinese reader route preserves the same projection, count, archive, and non-claim semantics through localized Rust references; source report commands and bytes are not copied. |
| `docs/features/human-benefit-report.ja.md` | implemented-different-by-design | The Japanese reader route preserves the same deterministic projection and evidence boundary through localized Rust references; source report commands and bytes remain outside the target contract. |
| `docs/maintainers/task-outcome-events.md` | implemented-different-by-design | Tri-language Rust event references, the strict event model, and regressions cover append-only history, correction/supersession, fingerprints, relationships, privacy, and provider-evidence boundaries. Source Python generator/validator/renderer files are semantic material only. |
| `docs/operations/quality-gates.md` | implemented-different-by-design | Rust Contract-aware CI gates, governance-integrity checks, the reviewed gate manifest, CI/release surfaces, and runner tests preserve dynamic profiles, shadow comparison, evidence ownership, timeout, performance-sample, and traceability responsibilities. Source `make quality`, `Makefile.ai.stack`, and Python runner bytes remain adopter/provider boundaries. |
| `docs/operations/quality-gates.zh-CN.md` | implemented-different-by-design | The Chinese CI references and gate manifest preserve the source quality hierarchy, dynamic route, shard/evidence, timeout, performance, and traceability semantics with explicit `--repo`; source Make/Python configuration is not installed into adopters. |
| `docs/operations/quality-gates.ja.md` | implemented-different-by-design | The Japanese CI references and gate manifest preserve the source quality hierarchy, dynamic route, shard/evidence, timeout, performance, and traceability semantics with explicit repository context; source Make/Python configuration is not copied. |

No implementation omission was found. The target intentionally places these
responsibilities under `docs/features`/`docs/reference` and typed Runtime/gate
surfaces rather than creating source-only `docs/maintainers` or
`docs/operations` files. Missing same-path files are therefore an explicit
layout boundary, not an unreviewed omission. Contract intent and acceptance
criteria stay in their authored language; localization changes presentation
only and never governance facts.

The shared Runtime is installed once outside each adopter. Every attached
object/adopter repository inherits its own `.ai/`, Contract, evidence,
knowledge, and adapter context through explicit `--repo`; source Python
modules, Make targets, report files, and quality configuration are not copied.
The WI-475 ledger records all seven changed paths with prior-classification and
source-change provenance and removes their deferred status. It now contains
4,262 `generated-history`, 303 `implemented-different-by-design`, one
`implemented-equivalent`, four `not-applicable`, 66 `reference-only`, and 483
`deferred-next-batch` records; `migrate-gap` remains zero.

## WI-482 — lifecycle, parallel, and trust-layer reference comparison

WI-482 re-reads the eight paths changed between the previous comparison and
the maintained local reference commit
`fde3380f81fea5fd2e288f7a8849f737dc074060`. The current Rust baseline is
`origin/main` `1f65a3b8bf09e54d4f9600fc5d64d8bbcb3ed62f`; comparison checks use
the published `ai-cockpit 0.2.57` binary
(`f03a13251a6fe57783528efbeae6ddd23bc2cc31dd2a1501d5421aac169a1d58`).
The source changes narrow a short lifecycle route, move parallel/handoff detail
to its dedicated reference, remove a template-only quality-shard section, and
remove an obsolete `REPORT_LANGUAGE` example argument. No source Python,
Makefile, provider configuration, or same-path documentation is copied.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/operations/work-item-lifecycle.md` | implemented-different-by-design | `docs/reference/agent-workflow.md`, `docs/reference/outcome-report.md`, and this comparison route carry the Rust-native lifecycle, human pause, exact cleanup, and current-Work-Item boundary. |
| `docs/operations/work-item-lifecycle.zh-CN.md` | implemented-different-by-design | The Chinese agent workflow and Outcome routes preserve the same lifecycle and stop rules with explicit `--repo`; source wording and Make commands are not installed. |
| `docs/operations/work-item-lifecycle.ja.md` | implemented-different-by-design | The Japanese agent workflow and Outcome routes preserve the same lifecycle and stop rules with explicit repository context; source wording and Make commands are not copied. |
| `docs/reference/agent-parallel-work-items.md` | implemented-different-by-design | `docs/reference/cross-work-item-dedup.md`, `docs/reference/affected-verification.md`, `docs/reference/agent-workflow.md`, `AGENTS.md`, and `.ai/README.md` keep dedicated-worktree, scope, evidence, serialization, and visible-handoff boundaries. Agent conversation delivery remains an adapter responsibility. |
| `docs/reference/ai-cockpit-work-item-lifecycle.md` | implemented-different-by-design | Rust lifecycle and pre-finish boundaries are documented in `docs/reference/agent-workflow.md`, `docs/reference/outcome-report.md`, `docs/reference/ci-quality-gates.md`, and the Runtime. Template-only pytest shards and `REPORT_LANGUAGE` are not target requirements. |
| `docs/trust-layer.md` | implemented-different-by-design | `docs/philosophy.md`, `docs/security/enterprise-governance.md`, `docs/architecture.md`, and `docs/capabilities.md` preserve trust-chain, delegated-evidence, human-decision, and limitation semantics with Rust-native boundaries. |
| `docs/trust-layer.zh-CN.md` | implemented-different-by-design | The Chinese philosophy, enterprise-governance, architecture, and capabilities routes preserve the same trust semantics and explicit external-provider boundary. |
| `docs/trust-layer.ja.md` | implemented-different-by-design | The Japanese philosophy, enterprise-governance, architecture, and capabilities routes preserve the same trust semantics and explicit external-provider boundary. |

No implementation omission was found in these changed paths. The target
intentionally has no same-path `docs/operations/*`, `docs/trust-layer.*`, or
agent-handoff appendix: accepted responsibilities are split across
Rust-native reader routes and the explicit Agent adapter boundary. Contract
intent and acceptance criteria remain in their authored language; localization
changes presentation only. The ledger records all eight paths as
`implemented-different-by-design`, preserves source-change history, and reduces
the deferred set to 475 records.

## WI-494 — capability, comprehension, and deprecated-assets rebaseline

WI-494 re-reads the seven source paths whose bytes changed after their earlier
reference-only decisions. The current source records remain bounded to the
reference template: a capability claim matrix, three anonymized participant
responses, a revision-bound comprehension result (JSON and report), and a
source-specific deprecated-assets registry.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/capability-truth-matrix.json` | reference-only | Typed request-scoped capability/status projections and this parity ledger; source freshness, template status, and adopter/provider claims are not imported into the Runtime protocol. |
| `docs/reference/comprehension-validation-responses/peter_01.en.json` | reference-only | English reader and human-benefit/Outcome routes explain the boundary; anonymized participant bytes remain source-study evidence only. |
| `docs/reference/comprehension-validation-responses/tanaka_01.ja.json` | reference-only | Japanese reader route preserves the documentation boundary; participant response data is not target evidence. |
| `docs/reference/comprehension-validation-responses/xiaoli_01.zh-CN.json` | reference-only | Simplified Chinese reader route preserves the documentation boundary; participant response data is not target evidence. |
| `docs/reference/comprehension-validation-results.json` | reference-only | Human-benefit and comprehension guidance preserves revision binding and narrow claims; the source receipt is not a Rust product/release/safety/enterprise claim. |
| `docs/reference/comprehension-validation-results.md` | reference-only | Rust Outcome and reader references describe human-facing evidence without copying the source study report or inheriting its claim. |
| `docs/reference/deprecated-assets-registry.json` | reference-only | Immutable Work Item history, explicit resource finalization, and reviewed cleanup receipts provide the Rust boundary; the source scanner/registry is not deletion authority. |

No implementation omission was found. These source changes refresh
source-owned records and do not add a portable Runtime contract. The target
therefore records all seven as `reference-only`, preserving prior decisions
and source-change provenance in the append-only ledger. The ledger now contains
4,262 `generated-history`, 311 `implemented-different-by-design`, one
`implemented-equivalent`, four `not-applicable`, 73 `reference-only`, and 468
`deferred-next-batch` records; `migrate-gap` remains zero. The same boundary is
recorded in the Chinese and Japanese routes.

## WI-496 — distribution, profiles, multilingual assessment, and pre-release audit

WI-496 re-reads ten paths at the pinned local reference commit. Each decision
is semantic and file-specific; source Python/Make implementation, source
planning metadata, revision-bound assessment receipts, and provider release
claims are not copied into Rust authority or adopter state.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/distribution.md` | implemented-different-by-design | Rust release/distribution, installation, checksums, SBOM/provenance, and public/N-1 adopter acceptance routes. |
| `docs/reference/distribution.ja.md` | implemented-different-by-design | Japanese release and installation routes plus the same immutable artifact and adopter boundaries. |
| `docs/reference/documentation-context-registry.json` | reference-only | `.ai/README.md`, `.ai/glossary.md`, `AGENTS.md`, and documentation acceptance provide current-instruction/history boundaries; source plan metadata is not a portable protocol. |
| `docs/reference/governance-profiles.md` | implemented-different-by-design | Rust dynamic quality route, gate manifest, and typed governance controls; VerificationTier and EvidenceAssurance remain orthogonal. |
| `docs/reference/governance-profiles.zh-CN.md` | implemented-different-by-design | Chinese Rust quality-route and governance-control documentation with explicit repository context. |
| `docs/reference/governance-profiles.ja.md` | implemented-different-by-design | Japanese Rust quality-route and governance-control documentation with explicit repository context. |
| `docs/reference/japanese-capability-assessment.json` | reference-only | Source's 58-file, revision-bound assessment receipt is not transferable; target maintains its own bounded multilingual evidence. |
| `docs/reference/japanese-capability-assessment.md` | implemented-different-by-design | Tri-language Japanese capability boundary, localized Outcome tests, and explicit non-fluency claims. |
| `docs/reference/pre-release-documentation-alignment.json` | reference-only | Source revision/work-item audit receipt is not portable evidence; target runs fresh documentation, parity, and governance gates. |
| `docs/reference/pre-release-documentation-alignment.md` | reference-only | Source pre-release report remains historical/reference-bound; target release evidence is generated independently. |

No implementation omission was found. The six portable responsibilities are
implemented through Rust-native release, governance, and multilingual reader
surfaces; four source-owned reports/registries remain reference-only. No
`migrate-gap` is introduced. Current counts are 3,681 `generated-history`,
273 `implemented-different-by-design`, 1 `implemented-equivalent`, 4
`not-applicable`, 73 `reference-only`, and 418 `deferred-next-batch`.

## WI-504 — reference documentation batch 29

WI-504 re-reads five changed reference paths at the pinned local commit
`fde3380f81fea5fd2e288f7a8849f737dc074060`. Four changes narrow
source/provider-specific documentation; the root `docs/upgrade.md` change is a
reader-entry compatibility pointer. Rust preserves the portable governance
meaning through its native Runtime and tri-language routes without copying
source Python, Make, provider commands, or source evidence.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/repository-workflow.ja.md` | implemented-different-by-design | The Rust Japanese workflow already omits the removed `REPORT_LANGUAGE` argument and documents explicit repository-scoped lifecycle, evidence, review, and cleanup. |
| `docs/reference/troubleshooting.md` | implemented-different-by-design | The Rust tri-language troubleshooting route keeps the general stop/recovery and evidence-preservation contract; provider-specific handoff records remain external. |
| `docs/reference/verification-evidence-reuse.md` | implemented-different-by-design | The source no-change decision is specific to its Python/Make proposal; Rust's separately authorized reuse remains bounded by identity, snapshot, policy, and fail-closed validation. |
| `docs/reference/work-item-lifecycle-closure.md` | implemented-different-by-design | Rust-native closure, exact cleanup, and recovery routes retain the portable boundary; source hosted-governance/Make recovery details are not Runtime commands. |
| `docs/upgrade.md` | implemented-different-by-design | A minimal root compatibility entry points to the canonical Rust tri-language upgrade reference, preserving the reader route without duplicating implementation details. |

No `migrate-gap` was found. The source/provider-specific removals do not weaken
the target's governed lifecycle, reuse, or upgrade boundaries, and the new root
entry closes the only reader-navigation omission found in this slice. The
current snapshot is 3,681 `generated-history`, 278
`implemented-different-by-design`, 1 `implemented-equivalent`, 4
`not-applicable`, 73 `reference-only`, and 413 `deferred-next-batch` records.

## WI-507 — language-adaptation example reader boundary

WI-507 re-reads five maintained reference example README files one by one at
the pinned local commit `fde3380f81fea5fd2e288f7a8849f737dc074060`. These are
source/provider onboarding examples: they prescribe stack-specific installers,
Make presets, coverage patterns, and sample Contracts for application teams.
They are not Runtime code, provider evidence, or a portable JSON wire contract.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `examples/flutter/README.md` | reference-only | `docs/reference/flutter-fixture-adaptation.*`, `docs/reference/contract-fields.md`, and `docs/reference/verification-route.md` preserve explicit scope, owner-approved commands, evidence binding, and shared Runtime/adopter isolation. Flutter/Dart installation, Make presets, coverage YAML, application code, and source JSON are not copied. |
| `examples/go/README.md` | reference-only | `docs/getting-started/adopter-configuration.md`, `docs/reference/contract-fields.md`, and `docs/reference/verification-route.md` preserve generic Contract/verification/evidence boundaries. Go toolchain commands, Make presets, coverage patterns, and application examples remain adopter-owned. |
| `examples/java/README.md` | reference-only | `docs/getting-started/examples/java.md`, `docs/reference/contract-fields.md`, and `docs/reference/verification-route.md` preserve owner-declared scope, verification, evidence, and repository isolation. Gradle/Spring/Android commands, coverage presets, and sample code are not Runtime requirements. |
| `examples/kotlin/README.md` | reference-only | `docs/getting-started/adopter-configuration.md`, `docs/reference/contract-fields.md`, and `docs/reference/verification-route.md` preserve the generic governance boundary. Kotlin/Gradle commands and coverage patterns remain adopter/provider responsibilities. |
| `examples/php/README.md` | reference-only | `docs/getting-started/adopter-configuration.md`, `docs/reference/contract-fields.md`, and `docs/reference/verification-route.md` preserve explicit Contract scope, verification, evidence, and shared-Runtime isolation. Composer/PHPUnit/PHPStan commands and application paths are not copied. |

No implementation omission was found in this slice: the source files are
reference-only application onboarding material, and the portable meaning is
already expressed by existing Rust-native Contract, verification, evidence,
and adopter-boundary routes. No source stack, installer, Make command, or
sample Contract decision is inherited by this repository or its adopters.
The current 4,450-path reference set now contains 3,681 `generated-history`,
278 `implemented-different-by-design`, 1 `implemented-equivalent`, 4
`not-applicable`, 78 `reference-only`, and 408 `deferred-next-batch` records;
the append-only ledger retains 669 retired records and `migrate-gap` remains
zero.

## WI-508 — stack-adaptation example reader boundary

WI-508 re-reads five maintained reference example README files one by one at
the pinned local reference commit
`fde3380f81fea5fd2e288f7a8849f737dc074060`. These are source/provider
onboarding examples for Python, Ruby, Rust, Swift, and TypeScript. They show
stack-specific installers, quality commands, coverage patterns, and sample
Contracts; they are not Runtime code, provider evidence, or a portable JSON
wire contract.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `examples/python/README.md` | reference-only | Existing Python fixture-adaptation, Contract-field, and verification-route docs preserve owner-declared scope, commands, evidence, and repository isolation. Python installer, Make, coverage, and sample Contract/Summary decisions remain adopter-owned. |
| `examples/ruby/README.md` | reference-only | The adopter-configuration, Contract-field, and verification-route docs preserve the generic governance boundary. Bundler/RuboCop/RSpec or Rake commands, coverage, and application examples remain adopter/provider responsibilities. |
| `examples/rust/README.md` | reference-only | Adopter configuration, Contract fields, verification route, and CI-quality-gate docs preserve the project-owned Cargo and evidence boundary. The source inline-test caveat, Make preset, and sample decisions are not Runtime requirements. |
| `examples/swift/README.md` | reference-only | Existing iOS Swift fixture-adaptation, Contract-field, and verification-route docs preserve explicit calibration, evidence, and repository isolation. SwiftPM/Xcode commands, coverage, platform/signing assumptions, and sample decisions remain adopter/provider responsibilities. |
| `examples/typescript/README.md` | reference-only | Existing TypeScript fixture-adaptation, Contract-field, and verification-route docs preserve explicit commands, evidence binding, shared Runtime isolation, and human review. npm/Node scripts, dependencies, fixture lifecycle, coverage, and sample decisions remain adopter/provider responsibilities. |

No implementation omission was found in this slice. The portable meaning is
already represented by Rust-native Contract, verification, evidence, CI, and
adopter-boundary routes. The target and attached object repositories do not
inherit source stack installers, Make presets, application examples, or sample
Contract decisions. The current tracked set contains 3,681
`generated-history`, 278 `implemented-different-by-design`, one
`implemented-equivalent`, four `not-applicable`, 83 `reference-only`, and 403
`deferred-next-batch` records; `migrate-gap` remains zero. The append-only
ledger continues to retain 669 retired reference paths.

This is semantic/documentation parity, not Python/Ruby/Rust/Swift/TypeScript
toolchain support, source-command compatibility, or JSON-wire compatibility.
Each adopter installs one shared Runtime externally and binds its own facts,
Contract, evidence, knowledge, and Agent adapter with explicit `--repo`.

## WI-510 — installer entrypoint and wizard locale boundary

WI-510 reads four maintained reference files one by one at the pinned local
reference commit `fde3380f81fea5fd2e288f7a8849f737dc074060`. The source
installer and wizard locales are intentionally separated from the Rust
Runtime's immutable Release and repository-bound onboarding route.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `install.sh` (`14f157f828e3ba8d1dd0886708b7eae223fe6d08`) | implemented-different-by-design | `docs/getting-started/installation.*`, `docs/getting-started/installation-security.*`, `docs/release/distribution.*`, and immutable `tests/release/adopter_acceptance.sh` / `adopter_upgrade_acceptance.sh` preserve tagged-source selection, digest verification, cleanup, rollback, and isolation boundaries. Rust installs one shared binary Release and requires explicit `inspect`/`attach`/Agent actions; source Shell/Python installer bytes and implicit target writes are not copied. |
| `locales/wizard/en.json` (`1b9bfc3535e507c8478b071b641d974cb031e59e`) | reference-only | `docs/getting-started/installation.md`, `docs/reference/commands.md`, and `docs/reference/outcome-report.md` document the Rust presentation boundary. English Runtime labels and human Outcome sections are supported; source interactive-wizard prompt/session controls remain host or adapter responsibility. |
| `locales/wizard/ja.json` (`8fab9ba89bd2bac5ccd51e8cb70dfea719435f5c`) | reference-only | `docs/getting-started/installation.ja.md`, `docs/reference/commands.ja.md`, and `docs/reference/outcome-report.ja.md` document localized Japanese Runtime presentation. Rust does not ship a second interactive installer or infer approval from provider conversation controls. |
| `locales/wizard/zh-CN.json` (`591e11709864edf2846bfe63aab246b1dafd6473`) | reference-only | `docs/getting-started/installation.zh-CN.md`, `docs/reference/commands.zh-CN.md`, and `docs/reference/outcome-report.zh-CN.md` document localized Chinese Runtime presentation. The source wizard locale is not copied and cannot authorize repository changes. |

No implementation omission was found in this slice. The source `install.sh`
semantics are represented by the Rust public Release, checksum/SBOM/provenance,
explicit repository attachment, and isolated adopter acceptance boundaries.
The three locale files are source presentation assets, not portable governance
policy; Runtime-owned labels are localized while Contract facts remain in their
authoring language and host/Agent conversation UX remains external. The
current 4,450-path set contains 3,681 `generated-history`, 279
`implemented-different-by-design`, one `implemented-equivalent`, four
`not-applicable`, 86 `reference-only`, and 399 `deferred-next-batch` records;
the append-only ledger retains 669 retired paths and `migrate-gap` remains
zero.

## WI-516: release, adoption, calibration, and evidence batch

WI-516 re-read 17 current paths at the pinned source commit. The paths cover
release projections, Python development metadata, adopter evidence, archive,
baseline/cost observation, calibration, capability truth, and canonical
evidence. Each current path is classified `implemented-different-by-design`
with an explicit Rust counterpart or non-claim in the inventory. The retired
`scripts/ai_adoption_reality_report.py` path was checked against the retired
ledger and remains historical/non-current; it is not claimed as a Runtime
capability. This is semantic parity for the adopter boundary, not source
Python, packaging, provider-state, interactive-wizard, or JSON-wire copying.

This is semantic/documentation parity, not source installer, Python dependency,
interactive wizard, or JSON-wire compatibility. Each object/adopter repository
still installs the shared Runtime externally and inherits only the explicit
repository-bound attach, Agent, Contract, evidence, knowledge, and Outcome
boundaries.

## WI-521 — source guard and adoption-check batch 35

WI-521 re-read the next twelve current reference paths one by one at the pinned
local commit `fde3380f81fea5fd2e288f7a8849f737dc074060`. The slice covers
adoption readiness, archive recovery, backtrack/test weakening, governance cost
budgets, capability claims, coverage, provider bot intake, diff ownership,
guard calibration, and file-boundary guards. The retired
`tests/test_ai_check_backtrack.py` path is excluded from the current slice and
remains historical metadata only.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `scripts/ai_check_adoption_ready.py` | reference-only | `docs/getting-started/standard-adoption-guide.md`, `adopter-configuration.md`, and repository `status`/`doctor` facts preserve the adopter checklist. Source Makefile.ai, CODEOWNERS, SECURITY, CI, and production-readiness checks remain adopter/provider boundaries. |
| `scripts/ai_check_archive_recovery.py` | implemented-different-by-design | Append-only archive sequences, predecessor-bound recovery, and strict finalization validation in `crates/cockpit-repository` protect immutable ownership without copying source archive/traceability files. |
| `scripts/ai_check_backtrack.py` | implemented-different-by-design | Rust derives fail-closed test/coverage weakening and input-trust signals. Source report-only snapshot/work-item deletion warnings remain source maintenance projection. |
| `scripts/ai_check_budget_impact.py` | implemented-different-by-design | Typed identity-bound `PerformanceBaseline`/cost observations and explicit local budgets are advisory; source template metric thresholds and repayment records are not imported. |
| `scripts/ai_check_capability_claims.py` | reference-only | The source lexical claim/matrix checker remains source documentation tooling. Rust capability truth is observed, repository-bound, and explicit about exclusions; prose cannot silently become evidence. |
| `scripts/ai_check_coverage_guard.py` | implemented-different-by-design | Rust detects coverage weakening and requires declared verification evidence. Source association rules and missing-test-diff reports are adopter/source maintenance policy. |
| `scripts/ai_check_dependabot_intake.py` | not-applicable | Dependabot event identity and automatic-merge handling are provider-specific; Rust retains generic delegated evidence and explicit Work Item source binding. |
| `scripts/ai_check_diff_ownership.py` | reference-only | Rust enforces Contract scope/outOfScope, repository isolation, and immutable archive ownership at lifecycle gates. The source cross-Work-Item YAML ownership preview is not Runtime authority. |
| `scripts/ai_check_guard_calibration.py` | implemented-different-by-design | Rust validates repository-bound Project Profile, capability declarations, policy precedence, and explicit calibration facts; source YAML guard-map calibration is not copied. |
| `scripts/ai_check_guards.py` | implemented-different-by-design | Typed Contract, authority, input-trust, lifecycle, and repository-isolation boundaries replace source file-ownership/boundary manifests without installing a second guard system. |
| `tests/test_ai_check_archive_recovery.py` | implemented-different-by-design | Native archive-integrity and resource-finalization transition tests cover immutable ownership and predecessor-bound recovery. |
| `tests/test_ai_check_budget_impact.py` | implemented-different-by-design | Native verification/performance tests cover typed budgets, identity-bound observations, and exact reuse without copying source fixtures. |

No new portable implementation omission was found in this slice. The source
scripts that describe repository facts or lifecycle safety are represented by
Rust-native typed checks and reader-facing guidance; source-specific adoption,
provider, lexical-claim, and cross-Work-Item report surfaces remain explicit
external/reference boundaries. The current 4,450-path set now contains 3,681
`generated-history`, 304 `implemented-different-by-design`, one
`implemented-equivalent`, five `not-applicable`, 89 `reference-only`, and 370
`deferred-next-batch` records; the append-only ledger retains 669 retired paths
and `migrate-gap` remains zero.

This is semantic/documentation parity, not Python/Make command, provider-event,
source YAML, or JSON-wire compatibility. Every attached object/adopter project
inherits the same shared Runtime, explicit `--repo` context, repository-local
Contract/evidence/knowledge, and human Outcome boundary; it does not inherit
the reference project's source scripts or adopter-specific policy values.

## WI-543 — safe reference-ledger checking and source checker batch 37

WI-543 compared the seven maintained source checker modules below at the pinned
commit `fde3380f81fea5fd2e288f7a8849f737dc074060`. The source remains a
specification/behavior corpus; Python, Make, YAML, provider, and source JSON
wire implementations are not copied into the Rust Runtime.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `scripts/ai_check_task_outcome.py` | implemented-different-by-design | Typed OutcomeV2/TaskOutcomeReport, append-only events, localized human handoff, and archive binding cover the portable boundary; source report wire and lexical policy remain source-specific. |
| `scripts/ai_check_test_weakening.py` | implemented-different-by-design | Snapshot-based Rust weakening signals and fail-closed unknowns cover the portable boundary; source thresholds and maintenance report format remain source/provider policy. |
| `scripts/ai_classify_operation_impact.py` | implemented-different-by-design | Operation-time policy and scope evaluation provide explicit impact facts without inferring intent or importing the source report format. |
| `scripts/ai_close_work_item.py` | implemented-different-by-design | Typed lifecycle/finalization/ready-on-base gates enforce closure; provider PR operations and source runner orchestration remain external. |
| `scripts/ai_common.py` | implemented-different-by-design | JSON/Git/scope/redaction concerns are distributed across typed Core, Protocol, repository, and conformance services rather than a copied helper. |
| `scripts/ai_critical_domain_guards.py` | implemented-different-by-design | Typed operation, authority, prompt-injection, and evidence-forgery controls preserve fail-closed governance without promoting lexical classification to authority. |
| `scripts/ai_dependabot_intake.py` | not-applicable | Dependabot event identity and bot-branch intake are provider-specific; generic delegated evidence and source binding remain available. |

No portable implementation omission was found in this batch. Historical and
retired records remain append-only; only the current pinned path set is
eligible for new comparison decisions. Every attached adopter inherits the
same shared Runtime, explicit repository binding, isolated Contract/evidence/
knowledge, fail-closed lifecycle, and human Outcome handoff, not source
checkers or provider-specific policy.

## WI-548 — governance and boundary script batch 38

WI-548 compared the next thirteen maintained reference scripts one by one at
the pinned local commit `fde3380f81fea5fd2e288f7a8849f737dc074060`. The batch
covers derived-artifact authority, installation boundaries, doctor and
documentation routing, domain/evidence models, external identity/handoff,
final acceptance, and lightweight impact hints. The inventory records semantic
counterparts and explicit non-claims; it does not copy Python modules or source
JSON wire formats.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `scripts/ai_derived_artifacts.py` | implemented-different-by-design | Typed Contract/evidence/archive/Outcome projections keep derived views non-authorizing; no source registry is copied. |
| `scripts/ai_detached_uninstaller.py` | reference-only | Installed-lifecycle docs describe proposal, ownership, bounded removal, and evidence retention; Rust does not provide a detached uninstaller. |
| `scripts/ai_disable_enable.py` | reference-only | Repository attachment and request-scoped Runtime replace a global installer toggle; no disable/enable state file is claimed. |
| `scripts/ai_doctor.py` | implemented-different-by-design | Repository-bound Rust `doctor` reports protocol/runtime/compatibility and fail-closed diagnostics; provider toolchains remain adopter facts. |
| `scripts/ai_documentation_authority.py` | implemented-different-by-design | `.ai` read-set, current/reference routes, frontmatter, and documentation gates provide one authority route without a second registry CLI. |
| `scripts/ai_documentation_journey.py` | implemented-different-by-design | Tri-language current/getting-started/reference indexes and acceptance checks preserve the reader journey. |
| `scripts/ai_domain_model.py` | implemented-different-by-design | Typed Core/Protocol/repository lifecycle services own transitions, evidence, identity, and fail-closed decisions. |
| `scripts/ai_enterprise_control_evidence.py` | implemented-different-by-design | Enterprise assurance, expiry, retention, and delegated evidence remain explicit and cannot be inferred from local receipts. |
| `scripts/ai_evidence_dependencies.py` | implemented-different-by-design | Verification binds Work Item, repository, snapshot, Contract, profile, policy, command, stage, runner, and Runtime identity. |
| `scripts/ai_external_handoff.py` | implemented-different-by-design | Typed release/MCP/Outcome handoffs preserve digest-bound external responsibility without provider execution in Core. |
| `scripts/ai_external_identity.py` | implemented-different-by-design | Typed authority and delegated provider/enterprise evidence preserve assurance levels without authenticating a person locally. |
| `scripts/ai_final_north_star_acceptance.py` | implemented-different-by-design | Final replacement acceptance keeps the external-adopter/provider evidence boundary and explicit limitations. |
| `scripts/ai_impact_classifier.py` | implemented-different-by-design | Impact is derived from explicit Contract, scope, profile, and operation-time facts; unknown impact never weakens a route. |

No portable implementation omission was found in this batch. The detached
uninstaller and global disable/enable scripts remain deliberate
source/provider boundaries, not missing Runtime features. Attached object
repositories inherit the same shared binary, explicit `--repo` binding,
isolated Contract/evidence/knowledge, and human Outcome rules; they do not
inherit source installer state, Python registries, or adopter-specific policy
values.

## WI-550 — lifecycle and Outcome script comparison batch 39

WI-550 compared sixteen maintained reference scripts one by one at the pinned
local commit `fde3380f81fea5fd2e288f7a8849f737dc074060`. The batch covers finish
and report generation, status and lifecycle truth, input trust, observability,
recovery, multilingual presentation, PR handoff, and required-evidence rules.
The inventory records semantic ownership and explicit boundaries; it does not
copy Python modules or source JSON wire formats.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `scripts/ai_finish.py` | implemented-different-by-design | Typed lifecycle, evidence, checkpoint, recovery, and human Outcome gates in repository services; source process/provider orchestration is not copied. |
| `scripts/ai_generate_human_report.py` | implemented-different-by-design | `OutcomeV2`/`TaskOutcomeReport` and visible renderer preserve report phases and evidence bindings; source report wire is not a Rust contract. |
| `scripts/ai_generate_status.py` | implemented-different-by-design | Repository-bound status/inspect/doctor projections replace generated `current_status.md` authority. |
| `scripts/ai_generate_task_outcome.py` | implemented-different-by-design | Typed task report/events provide deterministic findings, risks, interventions, evidence, and next action. |
| `scripts/ai_governance_compression.py` | implemented-different-by-design | Typed policy, operation-time evaluation, verification routing, and evidence controls own decisions; compression output remains presentation. |
| `scripts/ai_input_trust.py` | implemented-different-by-design | Request binding, untrusted-material evaluation, injection/forgery signals, and fail-closed decisions preserve trust semantics without source API compatibility. |
| `scripts/ai_japanese_capability.py` | implemented-different-by-design | Rust-native tri-language Outcome/MCP projections, documentation checks, and conformance tests replace source self-assessment. |
| `scripts/ai_lifecycle_facts.py` | implemented-different-by-design | Typed status and readiness projections expose read-only lifecycle facts; no generated Python facts file is authoritative. |
| `scripts/ai_lifecycle_truth.py` | implemented-different-by-design | Immutable lifecycle, successor, recovery, finalization, and archive receipts are owned by Rust Protocol/repository services. |
| `scripts/ai_multilingual_semantic_parity.py` | implemented-different-by-design | Fixed Runtime chrome is localized while Contract bytes and governance facts remain locale-neutral and original. |
| `scripts/ai_observability.py` | implemented-different-by-design | Verification timing/reuse metrics and append-only TaskOutcomeEvent JSONL retain deterministic observability; generic source sink is not required Runtime API. |
| `scripts/ai_post_archive_recovery.py` | implemented-different-by-design | Typed recovery/finalization/close paths preserve immutable identity-bound recovery; hosted failure parsing remains external. |
| `scripts/ai_render_task_outcome.py` | implemented-different-by-design | Rust human Outcome renderer provides marker, evidence, unknowns, decisions, and next action without copying source Markdown code. |
| `scripts/ai_render_task_outcome_multilingual.py` | implemented-different-by-design | CLI/MCP expose the same en/zh-CN/ja human handoff; Contract acceptance text is never silently translated. |
| `scripts/ai_render_task_outcome_pr.py` | reference-only | PR summary formatting is provider-facing presentation; digest-bound Outcome/release handoff remains the Runtime boundary. |
| `scripts/ai_required_evidence.py` | implemented-different-by-design | Typed Contract required-evidence classes, delegated evidence, policy routing, and release/permission controls preserve the portable boundary; source rule identifiers are not universal Rust fields. |

No portable implementation omission was found in this batch. Attached
object/adopter repositories inherit the shared Runtime, explicit repository
binding, isolated Contract/evidence/knowledge, fail-closed lifecycle, and
human Outcome handoff; they do not inherit source Python registries, provider
policy values, or source wire formats.

## WI-539 — source governance checker comparison batch 36

WI-539 re-read the next ten maintained source checker modules one by one at
the pinned commit `fde3380f81fea5fd2e288f7a8849f737dc074060`. The comparison
records semantic responsibility and ownership; it does not copy Python,
Make, YAML, or source JSON wire formats into the Rust Runtime.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `scripts/ai_check_guidelines.py` | implemented-different-by-design | Typed Contract guidelines remain human-owned; completion is bound through numbered acceptance/evidence and does not invent an untyped `guidelinesCompliance` field. |
| `scripts/ai_check_pr.py` | implemented-different-by-design | Archive, recovery, scope, and evidence checks are distributed across typed lifecycle gates; PR identity and hosted checks remain provider evidence. |
| `scripts/ai_check_reference_impact.py` | reference-only | Source AST/text impact scanning stays source/provider tooling. Rust keeps operation-time scope safety and fail-closed unknowns; it does not claim caller, external-consumer, or monitoring inference. |
| `scripts/ai_check_registry.py` | implemented-different-by-design | Versioned gate manifests and typed receipts provide deterministic checker registration, deduplication, and explicit unavailable-gate reasons. |
| `scripts/ai_check_review_policy.py` | implemented-different-by-design | Contract/preflight and provider PR review carry review authority; no second YAML policy or report-only focus list is installed. |
| `scripts/ai_check_scope.py` | implemented-different-by-design | Repository-relative scope/out-of-scope, dependency, parallel-boundary, and snapshot checks are typed Runtime gates. |
| `scripts/ai_check_serial_order.py` | implemented-different-by-design | Predecessor, merged PR, closure, exact resource cleanup, and synchronized-base requirements are enforced by lifecycle and ready-on-base checks. |
| `scripts/ai_check_status.py` | implemented-different-by-design | Request-scoped typed status and human Outcome projections replace generated `current_status.md` as authority. |
| `scripts/ai_check_status_consistency.py` | implemented-different-by-design | Read-only status derives active/archive ownership and rejects ambiguity; Runtime has no silent generated-status repair authority. |
| `scripts/ai_check_summary.py` | implemented-different-by-design | Strict Contract, evidence, archive, and Outcome bindings cover the portable boundary without claiming source Summary JSON compatibility or inferring human claims. |

No portable implementation omission was found in this slice. The single
reference-impact scanner is explicitly `reference-only`, not an untracked Rust
gap: static caller and external-consumer facts require an adopter/provider or
human-owned evidence source, and unknown impact remains fail-closed. The other
nine responsibilities are represented by Rust-native typed Protocol,
repository lifecycle, gate-manifest, status, and Outcome boundaries.

The current 4,450-path set contains 3,681 `generated-history`, 313
`implemented-different-by-design`, one `implemented-equivalent`, five
`not-applicable`, 90 `reference-only`, and 360 `deferred-next-batch` records;
the append-only ledger retains 669 retired paths and `migrate-gap` remains
zero. Every attached object/adopter project inherits the shared Runtime,
explicit repository binding, isolated Contract/evidence/knowledge, fail-closed
lifecycle, and human Outcome boundary. It does not inherit source checkers,
provider policy values, or stack-specific commands.

## WI-552 — installation and upgrade script comparison batch 40

WI-552 re-read seventeen maintained installer/upgrade paths at the pinned
reference commit. Each source responsibility is accounted for below; source
Python modules, installer registries, and wire JSON are not copied into Rust.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `scripts/ai_install_facts.py` | implemented-different-by-design | `attach`, `inspect`, `compatibility`, `doctor`, and release/adopter manifests bind facts; source `.ai/install` is not copied. |
| `scripts/ai_install_plan.py` | implemented-different-by-design | Explicit attach, migration, and adapter plans provide the read-only boundary; no source wizard wire is added. |
| `scripts/ai_install_status.py` | implemented-different-by-design | `status`, `compatibility`, `migrate plan`, and `doctor` provide status without a generated source file. |
| `scripts/ai_install_wizard.py` | implemented-different-by-design | Explicit CLI confirmation and localized Outcome replace implicit TTY/provider orchestration. |
| `scripts/ai_installer_bootstrap.py` | implemented-different-by-design | `attach` and Work Item scaffolding create only minimum repository-owned skeletons. |
| `scripts/ai_installer_catalog.json` | reference-only | Discovery uses the strict manifest and CLI/MCP schemas; the source provider catalog would overclaim support. |
| `scripts/ai_installer_detection.py` | implemented-different-by-design | `inspect`, `observe`, `status`, `doctor`, profile, and compatibility expose facts without source mode/provider inference. |
| `scripts/ai_installer_evidence.py` | implemented-different-by-design | Release/adopter acceptance and adapter ownership bind actions, roots, manifests, and digests. |
| `scripts/ai_installer_managed_regions.py` | implemented-different-by-design | Typed Agent adapter ownership and regular-path checks replace source heuristics. |
| `scripts/ai_installer_ownership.py` | implemented-different-by-design | Repository-local adapter ownership and strict Protocol/profile records are explicit and non-authorizing. |
| `scripts/ai_installer_repository.py` | implemented-different-by-design | Shared Git observer and explicit `--repo` operations fail closed on dirty, ambiguous, or foreign state. |
| `scripts/ai_installer_transaction.py` | implemented-different-by-design | Atomic writes, locks, path validation, and explicit migration/adapter confirmation replace source transaction code. |
| `scripts/ai_installer_upgrade.py` | implemented-different-by-design | Immutable Runtime artifacts and typed schema compatibility/migration own upgrade boundaries. |
| `scripts/ai_upgrade_apply.py` | implemented-different-by-design | `migrate apply --approved` is an adjacent digest-bound repository migration; Runtime replacement remains external. |
| `scripts/ai_upgrade_conflict_report.py` | implemented-different-by-design | Compatibility plans and doctor safe actions expose conflicts without source auto-resolution. |
| `scripts/ai_upgrade_proposal.py` | implemented-different-by-design | Migration planning preserves historical bytes and requires explicit approval. |
| `scripts/install_ai_cockpit.py` | reference-only | Rust installs from immutable release artifacts; the Python launcher is not a Runtime fallback. |

No portable omission was found. Attached object/adopter repositories inherit
the same shared Runtime, explicit repository binding, isolated protocol/
evidence/knowledge, strict migration boundary, and human Outcome handoff; they
do not inherit source installer state, provider policy, or Python code.

## WI-557 — source governance and maintenance script comparison batch 41

WI-557 re-read thirteen deferred source scripts at the pinned reference commit.
Twelve responsibilities are represented by Rust-native typed Protocol,
repository, verification, or documentation boundaries. The fixed recovery
scenario registry remains `reference-only`: the current Runtime exposes
explicit recovery commands and evidence, but does not claim a generic source
scenario catalog. Source Python implementation and source JSONL/YAML wire
formats are not copied.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `scripts/ai_issue_log.py` | implemented-different-by-design | Typed task-outcome events, finding/risk fingerprints, append-only evidence, and localized Outcome reporting provide issue identity and redaction boundaries; source issue-log JSONL is not a Runtime wire contract. |
| `scripts/ai_linked_worktree_recovery.py` | implemented-different-by-design | Linked-worktree topology, finalization/recovery receipts, and exact branch/worktree cleanup are repository lifecycle responsibilities; foreign duplicate diagnosis remains read-only. |
| `scripts/ai_ownership.py` | implemented-different-by-design | Typed Agent adapter ownership, managed-region checks, repository identity, and fail-closed mutation replace the source ownership helper. |
| `scripts/ai_performance_budget.py` | implemented-different-by-design | Identity-bound PerformanceBaseline samples, cost observations, and regression budgets provide advisory measurement without weakening verification. |
| `scripts/ai_project_profile.py` | implemented-different-by-design | Strict repository profile declarations, observed facts, profile policy, and explicit operation mappings replace source YAML profile validation. |
| `scripts/ai_purge.py` | implemented-different-by-design | Evidence retention metadata and `evidence purge-plan` require export, protected paths, confirmation, and a deterministic plan; Runtime does not silently delete evidence. |
| `scripts/ai_readiness_policy.py` | implemented-different-by-design | `status`, `doctor`, compatibility, and dynamic verification routes expose calibrated/readiness facts without executing source policy probes. |
| `scripts/ai_recovery_usability.py` | reference-only | Recovery documentation, explicit `recover`, and human Outcome guidance preserve the portable boundary; a generic fixed scenario registry is source-specific and is not claimed. |
| `scripts/ai_review_readiness_policy.py` | implemented-different-by-design | Preflight/review gates and provider-bound PR evidence provide review readiness; no report-only source focus list is installed. |
| `scripts/ai_risk_policy.py` | implemented-different-by-design | Typed Contract/Outcome findings, residual-risk signals, and explicit human decisions provide the risk boundary without copying source policy fields. |
| `scripts/ai_rollback.py` | implemented-different-by-design | Immutable release identity, migration plans, and recovery receipts provide bounded rollback/restore evidence; source managed-region restore semantics are not portable Runtime authority. |
| `scripts/ai_safety_gate.py` | implemented-different-by-design | Operation-time policy and critical-domain guards require explicit target, scope, authority, freshness, trust, impact, and verified evidence before dangerous actions. |
| `scripts/ai_schema_migration.py` | implemented-different-by-design | Typed compatibility and migration plans/apply preserve historical bytes, require approval, and reject reverse/ambiguous transitions. |

No portable implementation omission was found in this slice. Attached
object/adopter repositories inherit the shared Runtime, isolated repository
context, Contract/evidence/knowledge records, and human Outcome boundary; they
do not inherit source issue logs, provider policy values, Python modules, or a
generic source recovery catalog. Source test files for these scripts remain a
later file-level comparison batch and are not silently treated as completed.

## WI-559 — onboarding, trust, verification, and recovery script comparison batch 42

WI-559 re-read twenty maintained source scripts at the pinned local reference
commit `fde3380f81fea5fd2e288f7a8849f737dc074060`. Each responsibility is
accounted for below as a semantic Rust projection; Python, Make, provider
workflows, and source JSON wire formats are not copied.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `scripts/ai_onboard.py` | implemented-different-by-design | Explicit shared-Runtime attach, inspect, status, doctor, and profile commands; calibration and approval remain human-owned. |
| `scripts/ai_prepare_hosted_verification.py` | reference-only | Source-specific hosted-snapshot exception; hosted, CI, and release evidence remain external in Rust. |
| `scripts/ai_project_doctor.py` | implemented-different-by-design | Typed `RepositoryObservation`, inspect/status/doctor, and profile projections provide deterministic repository facts. |
| `scripts/ai_projection_lease.py` | implemented-different-by-design | Repository-local concurrency boundaries, leases, scope-overlap checks, and bounded parallel verification. |
| `scripts/ai_provider_merge_state_recovery.py` | implemented-different-by-design | Typed finalization/recovery receipts and delegated provider evidence validate identity and ancestry without claiming the source provider workflow. |
| `scripts/ai_quality_architecture.py` | reference-only | Python AST implementation audit is source tooling; Rust quality uses Cargo, Clippy, workspace tests, and native gates. |
| `scripts/ai_resume_work_item.py` | implemented-different-by-design | Typed resume/synchronization history, predecessor closure evidence, recovery identity, and mandatory revalidation. |
| `scripts/ai_start.py` | implemented-different-by-design | Repository-bound scaffolding, duplicate reservation, base/branch/worktree identity, concurrency gates, and preflight. |
| `scripts/ai_start_receipt.py` | implemented-different-by-design | Contract base/scope/snapshot identity and lifecycle receipts replace the source Start Receipt wire schema. |
| `scripts/ai_task_event_log.py` | implemented-different-by-design | Typed append-only `TaskOutcomeEvent` records, fingerprints, redaction, and archive/Outcome bindings. |
| `scripts/ai_terminology.py` | implemented-different-by-design | Typed policy, Outcome decision states, and tri-language glossary; verification tier and assurance remain orthogonal. |
| `scripts/ai_trust_guards.py` | implemented-different-by-design | Typed operation, intent, scope, authority, unknown, and human-review evaluation fail closed on ambiguity. |
| `scripts/ai_trust_schema.py` | implemented-different-by-design | Serde typed records, deny-unknown-fields validation, and Rust-native trust tests. |
| `scripts/ai_uninstall_facts.py` | implemented-different-by-design | Adapter ownership, agent doctor/detach/repair, repository identity, and retention metadata. |
| `scripts/ai_uninstall_proposal.py` | implemented-different-by-design | Explicit detach/purge plans, ownership/drift checks, evidence retention, and human authorization. |
| `scripts/ai_unknown_confirmation.py` | implemented-different-by-design | Identity-bound preflight human-decision requests with explicit unknowns, scope/evidence digests, and expiry. |
| `scripts/ai_validate_java_runtime.py` | reference-only | Java/JAVA_HOME selection is a stack-specific adopter/provider responsibility. |
| `scripts/ai_verification_context.py` | implemented-different-by-design | Request-scoped snapshot/observation, Contract/Summary bindings, changed paths, impact, and cached facts. |
| `scripts/ai_verification_policy.py` | implemented-different-by-design | Dynamic Tier/Assurance planning, stage and dependency routing, reuse, and evidence contexts. |
| `scripts/ai_verify.py` | implemented-different-by-design | Rust verify routes, checker registry, planner, Contract gates, and delegated release/adopter evidence. |

No portable implementation omission was found in this batch. The three
`reference-only` paths are source/provider or stack-specific tooling, not
missing Runtime capabilities. Every attached object/adopter repository
inherits the shared Runtime, explicit repository binding, isolated
Contract/evidence/knowledge, trust and lifecycle gates, and human Outcome
handoff; it does not inherit source launchers, provider commands, or Python
wire formats.

## WI-563 — wizard, intelligence, bootstrap, quality, and release checker comparison batch 43

WI-563 re-read the next twenty maintained source scripts one by one at the
pinned local reference commit `fde3380f81fea5fd2e288f7a8849f737dc074060`.
The table records each responsibility, its Rust counterpart, and the boundary
where the source remains presentation- or provider-specific. No Python,
Shell, Make, or source JSON implementation is copied.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `scripts/ai_wizard_io.py` | reference-only | TTY input primitives are host/Agent presentation concerns. Explicit non-interactive CLI/MCP schemas and visible Outcomes replace a second Runtime wizard. |
| `scripts/ai_wizard_localization.py` | implemented-different-by-design | CLI/MCP localize Runtime chrome and preserve authored Contract text; source locale resources and placeholder API are not wire requirements. |
| `scripts/ai_work_item_intelligence.py` | implemented-different-by-design | Typed Protocol, repository, knowledge, and CLI projections provide fact-derived, request-scoped Work Item intelligence with append-only evidence; source global cache/aggregation is not copied. |
| `scripts/ai_work_item_intelligence_benchmark.py` | reference-only | Source percentile benchmark output is implementation-specific. Rust performance samples and regression gates remain advisory and cannot authorize governance. |
| `scripts/ai_work_item_status.py` | implemented-different-by-design | Repository-bound `status` and `work-item status` provide stable JSON and human projections; no generated Python status file is authoritative. |
| `scripts/bootstrap_repository.py` | implemented-different-by-design | Shared Git observation plus `inspect`/`observe`/`status`/`doctor` provide remote, branch, dirty, conflict, and identity facts without source snapshot wire compatibility. |
| `scripts/bootstrap_wizard.py` | reference-only | Interactive Bootstrap session state is a presentation adapter. Rust keeps explicit detect/propose/confirm/attach commands and never manufactures readiness or authority. |
| `scripts/bootstrap_write_boundary.py` | implemented-different-by-design | Typed attach/migration/adapter writes enforce allowlists, regular paths, symlink rejection, atomic ownership, confirmation, and drift checks; source Makefile block protocol is not copied. |
| `scripts/check_bandit_baseline.py` | not-applicable | Python/Bandit baseline tooling has no Rust Runtime product surface; Cargo, Clippy, and Rust tests are the applicable controls. |
| `scripts/check_changed_critical_coverage.py` | implemented-different-by-design | Reviewed CI gate manifests and Contract/verification controls bind changed-critical coverage and candidate snapshots; the source pytest predictor/report is not used as authority. |
| `scripts/check_ci_release_evidence.sh` | implemented-different-by-design | Release/adopter harnesses validate artifact fields, digests, SBOM/provenance, and isolation against immutable public releases; source shell checker is not a fallback. |
| `scripts/check_critical_coverage.py` | reference-only | Python per-file coverage floors are source-specific. Rust keeps applicable package/test and performance gates without claiming that threshold or report wire. |
| `scripts/check_deprecated_assets.py` | reference-only | Source deprecated-asset registry is not a deletion authority. Rust uses immutable history, finalization, retention metadata, and owner-approved cleanup. |
| `scripts/check_dev_tool_versions.py` | implemented-different-by-design | Cargo lock/toolchain metadata and pinned CI actions provide reproducibility; Python package-pin parsing remains source-specific. |
| `scripts/check_docs_metadata.py` | implemented-different-by-design | Documentation acceptance and closed-Work-Item promotion check front matter, links, tri-language parity, command evidence, and claims without copying the source checker schema. |
| `scripts/check_governance_complexity.py` | implemented-different-by-design | Governance-integrity gates, complexity budgets, archive checks, and Runtime lifecycle evidence preserve the responsibility; source Python metrics cannot rewrite history. |
| `scripts/check_instruction_traceability.py` | implemented-different-by-design | Typed Contract/evidence/archive manifests and governance-integrity checks bind instruction → plan → implementation → acceptance links without adopting source audit JSON. |
| `scripts/check_pre_release_documentation_alignment.py` | implemented-different-by-design | Tri-language docs checks, projection promotion, and release gates provide current alignment; source revision-bound reports remain historical evidence. |
| `scripts/check_real_absurd_injection_docs.py` | implemented-different-by-design | Adversarial documentation and Rust trust regressions preserve explicit refusal evidence; source assessment helpers and fixed case registry are not Core code. |
| `scripts/check_release_distribution.py` | implemented-different-by-design | Immutable tag/archive discovery, checksum/SBOM/provenance, installer behavior, and post-release adopter acceptance are handled by Rust release workflows and harnesses. |

No portable implementation omission was found in this batch. Five
`reference-only` paths are presentation-, Python-coverage-, or source-registry
specific, and one Bandit checker is not applicable. The remaining fourteen
responsibilities are represented by Rust-native Runtime, repository, CI,
documentation, or release boundaries. Attached object/adopter repositories
inherit those same shared Runtime, explicit repository-context, isolated
Contract/evidence/knowledge, trust/lifecycle, and human Outcome boundaries;
they do not inherit source Python modules, provider policy values, or source
wire formats. The current 4,450-path set contains 3,681
`generated-history`, 403 `implemented-different-by-design`, one
`implemented-equivalent`, 7 `not-applicable`, 104 `reference-only`, and 254
`deferred-next-batch` records; `migrate-gap` remains zero and 669 retired
records remain append-only.
