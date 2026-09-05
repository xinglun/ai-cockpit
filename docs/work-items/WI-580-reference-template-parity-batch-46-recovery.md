---
author: AI Cockpit maintainers
title: "WI-580 — reference template parity batch 46"
description: "Re-read the remaining sixteen reference template paths and record bounded Rust semantic decisions without copying source implementation."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-580-reference-template-parity-batch-46-recovery
lastVerifiedBy: WI-580-reference-template-parity-batch-46-recovery
---


> This is an independent replacement delivery for the immutable WI-579 attempt. WI-579 registered the parity rows after archive, so its delivery remains an auditable failed attempt. This Work Item starts from the reviewed default branch and registers the parity rows before verification; it does not rewrite WI-579 history.

[简体中文](WI-580-reference-template-parity-batch-46-recovery.zh-CN.md) · [日本語](WI-580-reference-template-parity-batch-46-recovery.ja.md)

# WI-580 — Reference template parity batch 46

## Objective

Read every remaining `templates/**` path in the pinned local reference
checkout at commit `fde3380f81fea5fd2e288f7a8849f737dc074060`. Record an
explicit Rust/repository-native semantic counterpart or a bounded
`reference-only` decision. This is semantic parity, not source implementation,
Make target, stack command, or JSON-wire migration.

## File-level decisions

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `templates/agents/AI_COCKPIT_RULES.md` | implemented-different-by-design | `AGENTS.md`, `.ai/README.md`, `.ai/glossary.md`, `crates/cockpit-agent/src/lib.rs`, and the tri-language agent workflow preserve repository binding, Contract-first review, pause rules, evidence, Outcome, and exact cleanup. The template Markdown/Make surface is not copied. |
| `templates/glossary.md` | implemented-different-by-design | `.ai/glossary.md`, `docs/reference/commands.md`, and `docs/reference/agent-workflow.md` carry the governance vocabulary; project-domain placeholders remain adopter-owned and are not invented. |
| `templates/make/Makefile.ai` | implemented-different-by-design | Rust CLI/Repository/Verification services and the reviewed gate manifest provide the lifecycle, quality, and evidence responsibilities. Source Make/Python target names and shell defaults remain adopter/provider integration choices. |
| `templates/stacks/android.mk` | reference-only | Stack-specific Gradle/Android commands are source-template convenience defaults. Adopters declare their own toolchain and verification argv; the shared Runtime does not infer or copy them. |
| `templates/stacks/csharp.mk` | reference-only | Stack-specific .NET commands remain adopter-owned; Runtime verification records explicit commands and evidence without shipping a C# preset. |
| `templates/stacks/flutter.mk` | reference-only | Flutter/Dart toolchain defaults remain source/adopter configuration, not Runtime governance. |
| `templates/stacks/generic.mk` | reference-only | The fail-closed generic placeholder is a source template onboarding aid; Runtime keeps missing project checks visible and does not manufacture commands. |
| `templates/stacks/go.mk` | reference-only | Go formatter/test/lint commands are adopter-owned delegated checks, not a portable Runtime contract. |
| `templates/stacks/java.mk` | reference-only | Java/JAVA_HOME and Gradle/Maven selection remain stack/provider facts; Runtime does not select or install a JDK. |
| `templates/stacks/kotlin.mk` | reference-only | Kotlin/Gradle defaults are source-template convenience and remain outside Core. |
| `templates/stacks/php.mk` | reference-only | PHP formatter/test/static-analysis commands are adopter-owned and explicitly declared. |
| `templates/stacks/python.mk` | reference-only | Python/Ruff/Pytest defaults are source-template development tooling; this Rust Runtime does not install or copy a Python environment. |
| `templates/stacks/ruby.mk` | reference-only | Ruby/Bundler/Rake commands remain adopter-owned delegated verification. |
| `templates/stacks/rust.mk` | reference-only | Cargo commands are valid adopter choices but are not copied as a stack preset; Runtime selects only the repository's declared, profile-authorized verification route. |
| `templates/stacks/swift.mk` | reference-only | Swift/SPM/Xcode assumptions are adopter/platform-specific; Runtime does not claim Xcode or CocoaPods coverage. |
| `templates/stacks/typescript.mk` | reference-only | npm formatter/test/lint defaults remain adopter-owned and are not inferred by the shared Runtime. |

## Boundary and adopter inheritance

The three implemented-different decisions preserve the portable governance
responsibilities through the shared external Runtime and repository-local
documentation. The thirteen stack files are intentionally reference-only:
their commands, toolchain versions, and platform assumptions cannot be safely
made universal. Every attached object/adopter repository inherits the shared
Runtime, explicit `--repo` context, isolated Contract/evidence/knowledge,
dynamic verification boundary, and human Outcome handoff. It does not inherit
source Python, Make, stack presets, provider policy values, or source wire
formats.

## Verification

- `python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --apply-wi579-batch`
- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `python3 tests/docs/reference_comparison_metadata_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `python3 tests/docs/work_item_status_consistency.py --repo .`
- `git diff --check`

