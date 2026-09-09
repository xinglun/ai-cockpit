---
author: AI Cockpit maintainers
title: "Collaboration invariant coverage"
description: "Maps the ten collaboration-language semantic invariants to existing automated test coverage, citing exact test files and functions and stating remaining boundaries precisely."
audience: [maintainer, reviewer, contributor]
status: current
authority: canonical
lastVerifiedBy: WI-756-p1-handoff-reconstruction
---

# Collaboration invariant coverage

This page answers, for each of the ten semantic invariants stated in
WI-679's collaboration language contract
(`docs/reference/collaboration-language-contract.md`, merged in PR #675),
the question: does an automated test already enforce this today, and if so,
which one? It exists to prevent duplicate test infrastructure (reuse first)
and to state honestly which invariants have no automated cross-check yet.

Every citation below was read directly from the current test source at the
time of writing; none is inferred from a test's name alone.

**This page supersedes the earlier WI-681 draft.** WI-681 was closed without
merging because a different concurrent agent's Work Item independently used
the same short id (`WI-681-wi674-doc-promotion`, merged in PR #677) while
this content was still in review — a real multi-agent WI-number collision,
not a content defect. This redelivery (WI-740) also corrects one assessment
from that draft: invariant 8 was originally marked "No automated test
found," but closer reading found an existing test that already covers its
core claim (see row 8 below). Invariants 5 and 9, which WI-681 named as
gaps, were independently closed by WI-682 (PR #679) and WI-710 (PR #702)
before this redelivery, so they are now marked "Yes" here as well.

## Coverage table

| # | Invariant | Automated today? | Test(s) | What it asserts |
| --- | --- | --- | --- | --- |
| 1 | Verification pass != full acceptance/merge authorization | **Yes** | `crates/cockpit-cli/tests/outcome_handoff.rs::default_lifecycle_commands_emit_localized_handoffs_without_changing_stdout_json` | Drives one Work Item through `finish` (green) -> `archive` (yellow, finalization not yet done) -> a recorded `Deleted` finalization -> `close` (requires an explicit `--human-decision`), asserting the handoff text and stable JSON at each step never collapse these into a single "done" state. |
| 2 | Empty record != no risk | **Yes** | `crates/cockpit-repository/tests/outcome_report.rs::human_renderer_does_not_infer_risk_absence_or_test_strength_from_empty_fields` | Renders an Outcome with empty risk/test-weakening sections and asserts the human text never states "no risk found" or an equivalent positive claim from the absence of data. |
| 3 | Unknown fact not completed by presentation layer | **Yes (partial)** | `crates/cockpit-core/tests/adversarial_v2.rs::multilingual_adversarial_corpus_binds_wording_as_data` | Runs `tests/adversarial/manifest.json`'s semantic cases (currently 15) through `evaluate()` and asserts the evaluated fact is bound to case data, not to phrasing. Covers unknown-handling only for the specific adversarial phrasings already present in the manifest, not exhaustively. |
| 4 | Historical record cannot become current pass/fail without basis | **Yes** | `crates/cockpit-repository/tests/outcome_report.rs::human_renderer_preserves_historical_and_superseded_distinctions`, `::archived_report_tamper_is_red_and_not_reprojected_as_verified` | Asserts a `runtime_historical` record keeps its historical wording (never "Repair the missing evidence" phrasing meant for a current failure) and that a tampered archived report renders red rather than being silently reprojected as verified. |
| 5 | Same fact consistent across CLI/MCP/summary/report | **Yes** | `crates/cockpit-cli/tests/cli_mcp_outcome_parity.rs::cli_subprocess_and_mcp_handler_agree_on_the_same_outcome` (added by WI-682, PR #679) | Spawns the real `ai-cockpit` CLI binary as a subprocess for the CLI side and calls `cockpit_mcp::handle_request_for_repo()` in-process for the MCP side, against the identical repository fixture and Work Item, and asserts the CLI's `work-item outcome --json` output and the MCP `work_item_outcome` tool's `structuredContent.outcome` are exactly equal (using a `RuntimeContext` reconstructed to match the exact binary under test). This is the true cross-process check that the earlier `crates/cockpit-mcp/tests/rpc.rs::*_with_cli_parity` tests (which only compare two in-process calls) did not provide. |
| 6 | Language change does not alter facts/authorization/consequences | **Yes** | `crates/cockpit-cli/tests/outcome_handoff.rs::default_lifecycle_commands_emit_localized_handoffs_without_changing_stdout_json`; `crates/cockpit-core/tests/adversarial_v2.rs::multilingual_adversarial_corpus_binds_wording_as_data` | The CLI test spawns the real binary with `AI_COCKPIT_LANGUAGE` set to `en`/`zh-CN`/`ja` in turn and asserts the machine-readable stdout JSON fields are byte-identical across languages while only the human-readable stderr text localizes. The adversarial corpus test additionally checks 5 wording variants per language per semantic case evaluate identically. |
| 7 | Displayed next step matches current Runtime state/policy | **Yes (bounded)** | `crates/cockpit-repository/tests/scenario_matrix_next_action.rs` (WI-741, PR #713); `crates/cockpit-cli/tests/collaboration_consistency.rs::displayed_option_state_and_runtime_transition_stay_consistent_through_resume` (WI-753) | WI-741 binds displayed next-action messages to the observed scenario matrix. WI-753 additionally asserts that the displayed human-decision option and each subsequent Runtime transition agree through checkpoint, interruption, and resume. Coverage is intentionally bounded, not a universal state oracle. |
| 8 | Authorization applicability is rule-and-record based across session switches | **Yes (bounded)** | `crates/cockpit-repository/tests/preflight_review.rs::bound_human_review_receipt_allows_checkpoint_but_not_stale_reuse`; `crates/cockpit-cli/tests/collaboration_handoff.rs::new_agent_reconstructs_handoff_from_runtime_records_without_conversation_history` (WI-756) | The preflight test proves reuse is allowed only while identity bindings and the snapshot match, then rejects stale reuse. WI-756 adds a fresh-subprocess reconstruction that carries the recorded authorization and persisted stop boundary forward without creating a new decision. Coverage does not simulate every possible caller-identity transition. |
| 9 | Every human-decision question names subject/impact/recovery condition | **Yes** | `crates/cockpit-repository/tests/contract_preflight.rs::assert_human_decision_request_is_complete`, exercised by `::scaffold_preflight_is_not_ready_and_records_human_review_requirements` and `::high_risk_scenario_coverage_stops_at_preflight_for_human_review` (added by WI-710, PR #702) | Asserts `what_happened`, `why_it_matters`, `question`, `resume_condition`, `options`, `recommended_option`, and `recommendation_reason` are all non-empty, that `recommended_option` names one of the offered options, and that every option's `id`/`label`/`effect` are non-empty — checked against two independently-triggered real `needs_human_confirmation` scenarios. |
| 10 | Summary may omit detail but not a blocker/key-unknown/required decision | **Yes (partial)** | `crates/cockpit-repository/tests/outcome_report.rs::human_renderer_does_not_infer_risk_absence_or_test_strength_from_empty_fields` (adjacent assertions in the same file) | Covers the "empty is not positive" half directly; a dedicated test asserting a *populated* blocker/unknown/decision is never dropped by summarization was not found separately from the lifecycle test in #1, which exercises this incidentally. |

## Reading this table

- **Yes** means an existing, passing test in this repository's own suite
  already enforces the invariant as stated, and this page cites the exact
  function.
- **Partial** means real coverage exists but does not reach every case the
  invariant states (see the note in that row).
- **Indirect** (invariant 7 only) means related state-transition assertions
  exist, but no test directly asserts the specific claim end-to-end.

## Remaining boundaries

Invariant 7 no longer has a named zero-coverage gap: WI-741 and WI-753 provide
bounded executable checks tied to observed Runtime states. WI-756 also provides
a bounded Section V handoff-reconstruction check from Runtime records, while
none of these checks is a universal oracle for every state, option, or caller
identity combination.
