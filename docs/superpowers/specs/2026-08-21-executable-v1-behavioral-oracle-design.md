# Executable V1 Behavioral Oracle Design

## Problem

The current conformance corpus is self-contained: `input.json` is evaluated by
the Rust core and compared with a checked-in `expected.json`. The manifest says
`runtimeInvoked: false`, and the layout test requires that value. This proves
offline regression stability, but it does not prove that the expected semantics
were produced by the locked V1 reference runtime.

## Boundary

V1 remains an external Specification / Behavioral Oracle / Evidence Reference.
No V1 source, schema, installer, Make runtime, or Python environment is copied
into the Rust runtime or an adopter repository. The released product remains a
Rust single binary. Python is used only to execute the external V1 checkout in
the dedicated conformance job.

## Architecture

`tests/conformance/v1_oracle.py` is a test-only adapter. It imports governance
primitives from a caller-supplied V1 checkout and evaluates the semantic facts
from each V2 fixture. It emits the same seven canonical fields used by the Rust
corpus: decision state, blockers, unknowns, safe actions, required checks,
authority, and outcome state.

`crates/cockpit-core/tests/v1_oracle.rs` owns the trust boundary. It requires an
explicit `AI_COCKPIT_V1_ROOT`, verifies that its Git HEAD equals
`tests/conformance/v1-reference.lock`, invokes the adapter without a shell, and
compares every canonical case against the checked-in expected semantics. An
unavailable interpreter, wrong checkout, malformed output, missing case, extra
case, or semantic mismatch is a test failure.

Normal workspace tests keep the offline Rust regression. A separate mandatory
CI job clones the exact locked V1 commit and runs the ignored executable-oracle
test. This prevents ordinary adopter/runtime use from acquiring a Python or
network dependency while making Gate B evidence executable in CI.

## Probe Sources

- Scope: V1 `ai_common.included` matching.
- Missing, stale, and contradictory evidence: V1 `ai_domain_model.DomainService`.
- Destructive authority: V1 `evaluate_operation_time_policy`.
- Repository prompt injection and malicious deletion: V1
  `evaluate_governance_request`.
- Cross-Work-Item evidence: V1 `ai_generate_work_item_status.build_status`.
- Unsupported completion: V1 `unsupported_claim_gate.evaluate_claim`.
- Invalid archive: V1 `ai_outcome_gate.validate_terminal_outcome`.
- Unknown provider result: V1 recovery semantics with non-consistent provider
  evidence.
- Test and coverage weakening: V1 test-weakening and coverage-guard analyzers.

The adapter may translate V2 fixture schema into V1 function arguments, but it
must not read `expected.json`. Canonical projection rules must be deterministic
and shared; case-specific code may select a V1 probe but may not hard-code the
seven-field expected result.

## Failure Semantics

The executable Oracle fails closed. It never silently falls back to offline
fixtures, a different V1 commit, source-text inspection, or a passing V1 unit
test that does not return the fixture's semantics. Offline and executable modes
are reported separately in the manifest and documentation.

## Acceptance

1. All fourteen canonical cases execute against the exact locked V1 commit.
2. The adapter never reads any V2 `expected.json` file.
3. All seven semantic fields are compared for every case.
4. A wrong V1 commit and a changed expected semantic both make the Oracle test
   fail.
5. CI provisions V1 only for the Oracle job; normal tests and the product remain
   V1/Python/network independent.
6. The three-language conformance documentation states the evidence boundary
   accurately.
