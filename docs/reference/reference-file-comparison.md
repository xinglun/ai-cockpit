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

<!-- reference-inventory-counts: total=5119 generated-history=4262 implemented-different-by-design=198 implemented-equivalent=1 not-applicable=3 reference-only=4 deferred-next-batch=651 migrate-gap=0 -->

At the pinned v0.2.33 comparison baseline, the ledger contains 5,119 records:
4,262 `generated-history`, 198 `implemented-different-by-design`, one
`implemented-equivalent`, three `not-applicable`, four `reference-only`, and 651
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
records above are Rust-native, explicitly bounded counterparts; the 651
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
| `docs/concepts/trust-layer.md` | implemented-different-by-design | `docs/architecture/product-boundary.md`, `docs/philosophy.md`, enterprise-governance, and the capability truth matrix define calibrated trust, fail-closed unknowns, human control, and explicit non-goals. |

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
