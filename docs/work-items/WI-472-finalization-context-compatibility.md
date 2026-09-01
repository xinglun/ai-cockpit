---
author: AI Cockpit maintainers
title: "WI-472 — finalization context compatibility"
description: "Treat pending provider finalization sentinels as provisional before finish and archive."
audience:
  - maintainer
  - reviewer
  - adopter
workItemId: WI-472-finalization-context-compatibility
status: implemented
authority: authorized
lastVerifiedBy: WI-472-finalization-context-compatibility
terminalArchive: .ai/work-items/archive/WI-472-finalization-context-compatibility.contract.json
terminalVerification: .ai/evidence/WI-472-finalization-context-compatibility.verification.json
terminalFinalization: .ai/decisions/WI-472-finalization-context-compatibility.finalize.json
terminalDecision: .ai/decisions/WI-472-finalization-context-compatibility.close.json
---

# WI-472 — finalization context compatibility

## Intent and boundary

Prevent a provider placeholder such as `pending:<stable-reference>` from being
mistaken for a complete resource-finalization plan. A Work Item must remain
recoverable until a reviewed provider resource is bound. This Work Item does
not rewrite WI-471 or any other historical bytes and does not change the
object repository.

## Scope

- Classify `pending:*` and `unknown` finalization context values as provisional.
- Fail closed at the existing `finish`/`archive` boundary and preserve active
  bytes on rejection.
- Add protocol and repository regression tests and document the rule in all
  supported languages.

## Acceptance

1. Pending provider context cannot pass `finish` or `archive`.
2. A complete reviewed context remains accepted by existing lifecycle tests.
3. Rejection does not move or rewrite active Work Item bytes.
4. Tests and reference documentation describe the same provisional boundary in
   English, Simplified Chinese, and Japanese.
5. WI-471 remains immutable and is recovered only through an explicit successor
   receipt after this fix is released.

## Verification

- `cargo test --locked -p cockpit-protocol --test resource_finalization`
- `cargo test --locked -p cockpit-repository --test archive_integrity`
- `cargo test --locked --workspace`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/docs/documentation_acceptance.sh`

## Recovery boundary

If a provider PR is not yet known, use an explicit provisional context and keep
the Work Item active. Bind the exact reviewed PR URL before verification,
finish, and archive; do not edit an immutable archived Contract to replace a
pending sentinel.
