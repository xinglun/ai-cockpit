---
author: AI Cockpit maintainers
title: WI-633 - Reference rebaseline batch 53
description: Re-read the next 60 source-changed reference paths without copying source implementation.
audience: [maintainer, reviewer, adopter]
workItemId: WI-633-reference-rebaseline-batch-53
status: implemented
authority: human:repository-owner
lastVerifiedBy: WI-633-reference-rebaseline-batch-53
terminalArchive: .ai/work-items/archive/WI-633-reference-rebaseline-batch-53.contract.json
terminalVerification: .ai/evidence/WI-633-reference-rebaseline-batch-53.verification.json
terminalFinalization: .ai/decisions/WI-633-reference-rebaseline-batch-53.finalize.json
terminalDecision: .ai/decisions/WI-633-reference-rebaseline-batch-53.close.json
---

# WI-633 - Reference rebaseline batch 53

This Work Item re-reads the next 60 non-history paths whose bytes changed at
the pinned local reference commit `a9224aed77b5c317b53c4551a9eec306d91ee330`.
The complete machine-readable decision is in
`tests/conformance/reference_file_inventory.json`; the explicit ordered set is
`WI633_REFERENCE_PATHS` in `tests/conformance/reference_file_inventory.py`.

## Result set

Forty-three paths are `implemented-different-by-design`: their portable
responsibilities are represented by the shared Rust Runtime, repository-native
tests, CI/release boundaries, or reader documentation. Seventeen paths are
`reference-only`: provider-generated release artifacts, source catalogs,
sharding/benchmark tooling, aggregate reports, and adopter feature-parity
fixtures are not portable Runtime authority or target wire contracts. No path
is `deferred-next-batch` or `migrate-gap` after this batch.

The exact 60 paths, in stable order, are:

```text
scripts/ai_close_work_item.py
scripts/ai_cross_wi_integration.py
scripts/ai_diff_bound_reuse.py
scripts/ai_environment_reuse.py
scripts/ai_evidence_binding.py
scripts/ai_evidence_dependencies.py
scripts/ai_finish.py
scripts/ai_generate_human_report.py
scripts/ai_generate_knowledge_record.py
scripts/ai_generate_status.py
scripts/ai_generate_task_outcome.py
scripts/ai_generate_work_item_status.py
scripts/ai_governance_cost.py
scripts/ai_install_plan.py
scripts/ai_installer_adopter_capability_manifest.py
scripts/ai_installer_catalog.json
scripts/ai_knowledge_projection_benchmark.py
scripts/ai_knowledge_query.py
scripts/ai_onboard.py
scripts/ai_outcome_gate.py
scripts/ai_parallel_verification.py
scripts/ai_performance_diagnosis.py
scripts/ai_post_archive_recovery.py
scripts/ai_render_task_outcome.py
scripts/ai_render_task_outcome_multilingual.py
scripts/ai_start.py
scripts/ai_verification_policy.py
scripts/ai_verification_runtime.py
scripts/ai_verify.py
scripts/check_changed_critical_coverage.py
scripts/check_governance_complexity.py
scripts/check_release_distribution.py
scripts/check_supply_chain.py
scripts/determine_governance_profile.py
scripts/end_to_end_adoption_validation.py
scripts/installer/legacy.py
scripts/quality_measurements.py
scripts/quality_shard_workspace.py
scripts/quality_test_manifest.py
scripts/release_archive.py
scripts/sync_published_release_projection.py
target/quality/project-test-aggregate/receipt.json
target/release-v0-5-69-provider-release/provider-release.receipt.json
target/release-v0-5-69-provider-release/public-assets-32286215124/ci-release-evidence.json
target/release-v0-5-69-provider-release/public-assets-32286215124/provenance.json
target/release-v0-5-69-provider-release/public-assets-32286215124/release-digests.json
target/release-v0-5-69-provider-release/public-assets-32286215124/release-source.json
target/release-v0-5-69-provider-release/public-assets-32286215124/release.json
target/release-v0-5-69-provider-release/public-assets-32286215124/sbom.json
target/release-v0-5-69-provider-release/public-assets-32286215124/v0.5.69.tar.gz
target/release-v0-5-69-provider-release/rehearsal-artifact-32284965833/release-rehearsal.json
target/release-v0-5-69-provider-release/rehearsal.receipt.json
templates/agents/AI_COCKPIT_RULES.md
templates/make/Makefile.ai
tests/test_adopter_feature_parity.py
tests/test_adoption_e2e.py
tests/test_ai_adoption_reality_report.py
tests/test_ai_archive_work_item.py
tests/test_ai_check_backtrack.py
tests/test_ai_check_summary.py
```

Source Python, Make, provider decisions, generated release bytes, and source
JSON wire formats are not copied. Attached object/adopter repositories inherit
the shared Runtime, explicit repository context, isolated Contract/evidence/
knowledge, dynamic verification, fail-closed lifecycle, and visible human
Outcome.

## Acceptance

Run the pinned inventory check, tri-language documentation count check, source
policy check, and the repository quality gate. The next semantic batch starts
only after this Work Item is merged, closed, and post-close documentation
promotion is current.

See also: [中文](WI-633-reference-rebaseline-batch-53.zh-CN.md) ·
[日本語](WI-633-reference-rebaseline-batch-53.ja.md).
