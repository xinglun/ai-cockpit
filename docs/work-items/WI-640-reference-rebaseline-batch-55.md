---
author: AI Cockpit maintainers
title: WI-640 - Reference rebaseline batch 55
description: Re-read sixty pinned reference paths one by one without copying source implementation.
audience: [maintainer, reviewer, adopter]
workItemId: WI-640-reference-rebaseline-batch-55
status: implemented
authority: human:repository-owner
lastVerifiedBy: WI-640-reference-rebaseline-batch-55
terminalArchive: .ai/work-items/archive/WI-640-reference-rebaseline-batch-55.contract.json
terminalVerification: .ai/evidence/WI-640-reference-rebaseline-batch-55.verification.json
terminalFinalization: .ai/decisions/WI-640-reference-rebaseline-batch-55.finalize.json
terminalDecision: .ai/decisions/WI-640-reference-rebaseline-batch-55.close.json
---

# WI-640 - Reference rebaseline batch 55

## Intent

Re-read the next bounded set of paths from the pinned local reference one at a
time, confirm the Rust/native counterpart or an explicit reference-only
boundary, and fix a portable omission in this Work Item if evidence finds one.
The reference is a specification and behavior corpus, not a directory to copy.

## Scope and boundaries

- Reference commit: `a9224aed77b5c317b53c4551a9eec306d91ee330`.
- Exactly sixty paths below, plus the machine inventory, metadata, tri-language
  comparison/parity pages, and this Work Item's three language pages.
- No Python, Shell, Make, provider policy, stack fixture, or source JSON wire
  bytes are copied. Object/adopter repositories and global Agent/MCP config are
  out of scope.
- Every attached repository inherits the shared external Runtime, explicit
  `--repo`, isolated Contract/evidence/knowledge, dynamic verification,
  fail-closed lifecycle, and visible human Outcome.

## Ordered path set

`tests/test_quality_measurements.py`
`tests/test_quality_scope.py`
`tests/test_quality_session.py`
`tests/test_quality_shard_workspace.py`
`tests/test_quality_telemetry.py`
`tests/test_quality_test_manifest.py`
`tests/test_quick_install_release.py`
`tests/test_readiness_policy.py`
`tests/test_real_absurd_injection.py`
`tests/test_real_absurd_injection_docs.py`
`tests/test_real_adopter_reference_validation.py`
`tests/test_recovery_usability.py`
`tests/test_reference_impact.py`
`tests/test_release_archive_contract.sh`
`tests/test_release_distribution.py`
`tests/test_release_preflight.py`
`tests/test_release_state_consistency.py`
`tests/test_release_workflow.py`
`tests/test_required_evidence.py`
`tests/test_rollback.py`
`tests/test_ruff016_compatibility.py`
`tests/test_schema_migration.py`
`tests/test_start_and_archive.py`
`tests/test_supply_chain.py`
`tests/test_sync_published_release_projection.py`
`tests/test_system_invariants.py`
`tests/test_task_outcome_ai_finish_integration.py`
`tests/test_task_outcome_generator.py`
`tests/test_task_outcome_markdown_renderer.py`
`tests/test_task_outcome_multilingual.py`
`tests/test_task_outcome_schema.py`
`tests/test_task_outcome_validator.py`
`tests/test_terminology.py`
`tests/test_test_weakening.py`
`tests/test_trust_guards.py`
`tests/test_trust_layer_demo.py`
`tests/test_trust_layer_docs.py`
`tests/test_trust_schema.py`
`tests/test_typescript_fixture.py`
`tests/test_uninstall_facts.py`
`tests/test_uninstall_proposal.py`
`tests/test_unknown_confirmation.py`
`tests/test_upgrade_apply.py`
`tests/test_upgrade_conflict_report.py`
`tests/test_upgrade_proposal.py`
`tests/test_verification_evidence.py`
`tests/test_verification_impact_graph.py`
`tests/test_verification_policy.py`
`tests/test_wizard_fixtures.py`
`tests/test_wizard_io.py`
`tests/test_wizard_localization.py`
`tests/test_work_item_intelligence.py`
`tests/test_work_item_intelligence_benchmark.py`
`tests/test_work_item_intelligence_integration.py`
`tests/test_work_item_lifecycle_closure.py`
`tests/test_work_item_lifecycle_timing.py`
`tests/test_work_item_state_machine.py`
`tests/test_workflows.py`
`.ai/knowledge/work-items/dependabot-lockfile-sync-20260826.json`
`.ai/knowledge/work-items/dependabot-ruff-0165-sync-20260902.json`

## Decision

Fifty-four paths are `implemented-different-by-design`: portable quality,
release, evidence, trust, recovery, lifecycle, and reader responsibilities are
represented by Rust Runtime, native tests, CI/release surfaces, or docs. Six
paths are `reference-only`: source-specific TypeScript/wizard fixtures and two
historical knowledge records. The exact counterpart and reason for every row is
in [`reference_file_inventory.json`](../../tests/conformance/reference_file_inventory.json).
No `migrate-gap` remains in this batch. A missing source-change marker on a
pre-existing deferred record is recorded as `false`, preserving the distinction
between “no prior digest claim” and “skipped”.

## Verification

Run the inventory, metadata, documentation, parity, status-consistency, diff,
and repository quality checks declared by the Contract. Review the generated
Outcome before finish/archive/close; release publication remains deferred until
all six semantic batches and their cleanup/promotions are complete.

See also: [中文](WI-640-reference-rebaseline-batch-55.zh-CN.md) ·
[日本語](WI-640-reference-rebaseline-batch-55.ja.md).
