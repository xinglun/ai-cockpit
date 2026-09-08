---
author: AI Cockpit maintainers
title: "Collaboration invariant coverage"
description: "Maps the ten collaboration-language semantic invariants to existing automated test coverage, citing exact test files and functions, and names the remaining gaps precisely."
audience: [maintainer, reviewer, contributor]
status: current
authority: canonical
lastVerifiedBy: WI-681-p1-invariant-coverage-mapping
---

# Collaboration invariant coverage

This page answers, for each of the ten semantic invariants stated in
WI-679's collaboration language contract
(`docs/reference/collaboration-language-contract.md`, not yet on the default
branch as of this Work Item; see PR #675), the
question: does an automated test already enforce this today, and if so,
which one? It exists to prevent duplicate test infrastructure (reuse first)
and to state honestly which invariants have no automated cross-check yet.

Every citation below was read directly from the current test source at the
time of writing; none is inferred from a test's name alone.

## Coverage table

| # | Invariant | Automated today? | Test(s) | What it asserts |
| --- | --- | --- | --- | --- |
| 1 | Verification pass != full acceptance/merge authorization | **Yes** | `crates/cockpit-cli/tests/outcome_handoff.rs::default_lifecycle_commands_emit_localized_handoffs_without_changing_stdout_json` | Drives one Work Item through `finish` (green) -> `archive` (yellow, finalization not yet done) -> a recorded `Deleted` finalization -> `close` (requires an explicit `--human-decision`), asserting the handoff text and stable JSON at each step never collapse these into a single "done" state. |
| 2 | Empty record != no risk | **Yes** | `crates/cockpit-repository/tests/outcome_report.rs::human_renderer_does_not_infer_risk_absence_or_test_strength_from_empty_fields` | Renders an Outcome with empty risk/test-weakening sections and asserts the human text never states "no risk found" or an equivalent positive claim from the absence of data. |
| 3 | Unknown fact not completed by presentation layer | **Yes (partial)** | `crates/cockpit-core/tests/adversarial_v2.rs::multilingual_adversarial_corpus_binds_wording_as_data` | Runs `tests/adversarial/manifest.json`'s semantic cases (currently 15) through `evaluate()` and asserts the evaluated fact is bound to case data, not to phrasing. Covers unknown-handling only for the specific adversarial phrasings already present in the manifest, not exhaustively. |
| 4 | Historical record cannot become current pass/fail without basis | **Yes** | `crates/cockpit-repository/tests/outcome_report.rs::human_renderer_preserves_historical_and_superseded_distinctions`, `::archived_report_tamper_is_red_and_not_reprojected_as_verified` | Asserts a `runtime_historical` record keeps its historical wording (never "Repair the missing evidence" phrasing meant for a current failure) and that a tampered archived report renders red rather than being silently reprojected as verified. |
| 5 | Same fact consistent across CLI/MCP/summary/report | **Partial — real gap** | `crates/cockpit-mcp/tests/rpc.rs::mcp_work_item_outcome_returns_explicit_human_handoff_with_cli_parity`, `::mcp_blocked_outcome_exposes_the_same_recovery_facts_as_cli` | These call `cockpit_mcp::handle_request_for_repo()` in-process (a library call) and assert its output against fixed expected strings. This proves the MCP handler's *own* output is internally consistent and matches the documented CLI wording, but it never spawns the actual `ai-cockpit` CLI binary in the same test and diffs the two live outputs. **No true cross-process CLI-subprocess-vs-MCP-handler parity test exists today.** |
| 6 | Language change does not alter facts/authorization/consequences | **Yes** | `crates/cockpit-cli/tests/outcome_handoff.rs::default_lifecycle_commands_emit_localized_handoffs_without_changing_stdout_json`; `crates/cockpit-core/tests/adversarial_v2.rs::multilingual_adversarial_corpus_binds_wording_as_data` | The CLI test spawns the real binary with `AI_COCKPIT_LANGUAGE` set to `en`/`zh-CN`/`ja` in turn and asserts the machine-readable stdout JSON fields are byte-identical across languages while only the human-readable stderr text localizes. The adversarial corpus test additionally checks 5 wording variants per language per semantic case evaluate identically. |
| 7 | Displayed next step matches current Runtime state/policy | **Indirect** | Same lifecycle test as #1, plus `crates/cockpit-repository/tests/status_projection.rs::status_projection_distinguishes_archived_from_valid_closed_decision` | These assert the next-action-relevant fields (lifecycle phase, blockers) at each real state transition, but no test exists that specifically asserts the rendered "next action" sentence matches an independently computed expected action for a broad matrix of states. |
| 8 | Authorization applicability is rule-and-record based across session switches | **No automated test found** | — | No test simulates two different callers/sessions reusing (or failing to reuse) a Receipt/decision based on identity-binding digests. This is a real gap; see `docs/reference/collaboration-scenario-matrix.json` SCN-010/SCN-011/SCN-022 for the documented/observed behavior this would need to check. |
| 9 | Every human-decision question names subject/impact/recovery condition | **No dedicated test found** | — | The `humanDecisionRequest` shape is produced by production code (`preflight`) and is documented in `docs/reference/agent-workflow.md`, but no test was found asserting all four required fields (what/why/options/question/resumeCondition) are always non-empty together. |
| 10 | Summary may omit detail but not a blocker/key-unknown/required decision | **Yes (partial)** | `crates/cockpit-repository/tests/outcome_report.rs::human_renderer_does_not_infer_risk_absence_or_test_strength_from_empty_fields` (adjacent assertions in the same file) | Covers the "empty is not positive" half directly; a dedicated test asserting a *populated* blocker/unknown/decision is never dropped by summarization was not found separately from the lifecycle test in #1, which exercises this incidentally. |

## Reading this table

- **Yes** means an existing, passing test in this repository's own suite
  already enforces the invariant as stated, and this page cites the exact
  function.
- **Partial** means real coverage exists but does not reach every case the
  invariant states (see the note in that row).
- **No automated test found** means exactly that: a search of `tests/` and
  each crate's `tests/` directory did not find one. It does not mean the
  Runtime necessarily violates the invariant — several are also protected
  structurally by production code the tests above exercise indirectly — only
  that no test would currently catch a regression.

## Known gaps and recommended next Work Items

1. **Invariant 5 (cross-entry-point parity)**: add a new integration test
   that spawns the real `ai-cockpit` CLI binary as a subprocess (the pattern
   already used in `outcome_handoff.rs`) and, in the same test, calls
   `cockpit_mcp::handle_request_for_repo()` against the identical repository
   fixture (the pattern already used in `rpc.rs`), then asserts the two
   outputs agree on every stable field. This combines two already-proven
   patterns rather than introducing new test infrastructure.
2. **Invariant 8 (authorization reuse across sessions)**: add a test that
   records a decision receipt, then simulates a second "session" (a fresh
   process invocation against the same repository) attempting reuse after
   (a) no change — must reuse — and (b) a Contract/snapshot digest change —
   must require a fresh decision. `crates/cockpit-repository/tests/` already
   has the `attach()` + `start_work_item_with_options()` fixture pattern to
   build this from.
3. **Invariant 9 (`humanDecisionRequest` completeness)**: add a focused
   assertion (likely alongside existing preflight tests in
   `crates/cockpit-repository/tests/`) that every `needs_human_confirmation`
   preflight result's `humanDecisionRequest` has non-empty
   `whatHappened`/`whyItMatters`/`options`/`question`/`resumeCondition`.
4. **Invariant 7 (next-action correctness)**: this is the least tractable to
   test generically, since "correct" depends on the full state matrix in
   `collaboration-scenario-matrix.json`. A pragmatic next step is to assert
   the next-action field against a subset of that matrix's `observed`
   scenarios rather than attempting a general oracle.

None of these four are implemented by this Work Item; each is a bounded,
independently deliverable follow-on that reuses an existing test pattern
rather than adding a new one, consistent with this initiative's own
direction to reuse existing test infrastructure.
