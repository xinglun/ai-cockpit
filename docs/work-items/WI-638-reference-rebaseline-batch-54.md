---
author: AI Cockpit maintainers
title: WI-638 - Reference rebaseline batch 54
description: Re-read the next 60 pinned reference paths one by one without copying source implementation.
audience: [maintainer, reviewer, adopter]
workItemId: WI-638-reference-rebaseline-batch-54
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-638-reference-rebaseline-batch-54
---

# WI-638 - Reference rebaseline batch 54

This Work Item compares the next sixty non-history paths in the pinned local
reference commit `a9224aed77b5c317b53c4551a9eec306d91ee330`. The source is a
specification corpus. Its Python, shell, Make, provider, fixture, and JSON
bytes are not copied into the Rust repository or into attached adopters.

The machine-readable ledger is
`tests/conformance/reference_file_inventory.json`. It records the exact source
path, previous decision, Rust counterpart, current classification, and reason.
The comparison is semantic: a different Rust command, typed record, test,
release boundary, or reader page is valid when it preserves the portable
responsibility without importing source-local authority.

## Result set

Fifty-five paths are `implemented-different-by-design`. Their portable
responsibilities are provided by typed Rust Runtime services, repository-native
tests, CI/release/adopter boundaries, or reader documentation. Five paths are
`reference-only`: the source cross-Work-Item aggregate, deprecated-asset and
comprehension records, install-plan wizard test, and Java fixture test remain
source/provider or fixture evidence rather than Runtime authority. No path is
`deferred-next-batch` or `migrate-gap` after this bounded review.

The exact ordered path set is:

```text
tests/test_ai_cross_wi_integration.py
tests/test_ai_diff_bound_reuse.py
tests/test_ai_environment_reuse.py
tests/test_ai_evidence_binding.py
tests/test_ai_generate_work_item_status.py
tests/test_ai_governance_cost.py
tests/test_ai_onboard.py
tests/test_ai_parallel_verification.py
tests/test_ai_performance_diagnosis.py
tests/test_ai_post_archive_recovery.py
tests/test_ai_verification_runtime.py
tests/test_ai_verify.py
tests/test_changed_critical_coverage.py
tests/test_ci_quality_orchestration.py
tests/test_core_gates.py
tests/test_deprecated_assets.py
tests/test_documentation_comprehension_results.py
tests/test_documentation_journey.py
tests/test_end_to_end_adoption_validation.py
tests/test_finish_e2e.py
tests/test_finish_process_cleanup.py
tests/test_governance_complexity.py
tests/test_governance_profile.py
tests/test_human_benefit_report.py
tests/test_implementation_knowledge.py
tests/test_install_plan.py
tests/test_installed_lifecycle_e2e.py
tests/test_installed_runtime_parity.py
tests/test_installer.py
tests/test_instruction_traceability.py
tests/test_japanese_adopter_lifecycle.py
tests/test_japanese_capability.py
tests/test_java_fixture.py
tests/test_knowledge_installer_parity.py
tests/test_knowledge_projection_benchmark.py
tests/test_knowledge_query.py
tests/test_lifecycle_truth_core_677.py
tests/test_lightweight_verification.py
tests/test_makefile.py
tests/test_multilingual_semantic_parity.py
tests/test_onboard_e2e.py
tests/test_operation_impact.py
tests/test_operation_time_policy_reevaluation.py
tests/test_outcome_gate.py
tests/test_outcome_lifecycle_rules.py
tests/test_ownership.py
tests/test_parallel_lifecycle_contract.py
tests/test_performance_budget.py
tests/test_plan_cleanup.sh
tests/test_pr_aggregate.py
tests/test_pre_release_documentation_alignment.py
tests/test_project_governance.py
tests/test_project_governance_journey.py
tests/test_project_profile.py
tests/test_projection_isolation.py
tests/test_provider_backed_governance_validation.py
tests/test_provider_merge_state_recovery.py
tests/test_purge.py
tests/test_quality_architecture.py
tests/test_quality_gate_architecture.py
```

## File-level decisions

The target counterparts and boundary rationale for every path are in the
inventory. The following grouping is a human-readable index of that ledger:

| Source area | Rust-native boundary | Decision |
| --- | --- | --- |
| Cross-Work-Item aggregate and deprecated/comprehension records | `docs/reference/cross-work-item-dedup.md`, governance-integrity and documentation-boundary pages | `reference-only`; source reports are not Runtime authority. |
| Diff, environment, evidence, verification, outcome, lifecycle, and recovery tests | `crates/cockpit-*`, repository tests, and `docs/reference/*` | `implemented-different-by-design`; typed request-scoped identity and fail-closed evidence replace Python helpers. |
| Onboarding, installer, Japanese, Java, and adopter tests | attach/agent/release acceptance, fixture-boundary and getting-started pages | Explicit `--repo`, immutable release, and owner-declared toolchain facts are retained; source wizard/fixture bytes are not copied. |
| Knowledge, status, project profile, ownership, and projection tests | Knowledge crate, project governance, status/outcome projections, and isolation tests | `implemented-different-by-design`; generated projections remain repository-local and evidence-bound. |
| Quality, performance, operation-impact, purge, and release tests | dynamic gate manifest, verification planner, performance/adopter harnesses, and enterprise retention pages | `implemented-different-by-design`; cost is advisory and external/provider assurance remains delegated. |

No classification grants authority merely because a file exists. Unknown,
unavailable provider facts, and source-specific fixture claims remain visible as
unknown or external boundaries. Attached object/adopter repositories inherit the
same shared Runtime, explicit repository context, isolated Contract/evidence/
knowledge, dynamic verification, fail-closed lifecycle, and human Outcome; they
do not inherit source Python/Make implementation or source wire formats.

## Acceptance

Before verification, run the pinned inventory check, the documentation and
parity checks, and the repository quality gate. The next batch starts only
after this Work Item is merged, closed, post-close documentation promotion is
current, and the exact branch/worktree are removed. The release is intentionally
deferred until all six batches in this wave are complete; no intermediate
release is implied by this Work Item.

See also: [中文](WI-638-reference-rebaseline-batch-54.zh-CN.md) ·
[日本語](WI-638-reference-rebaseline-batch-54.ja.md).
