# SDD ledger — plan: ../admission-20261016/approved-proposal-v2.zh-CN.md

## Binding and constraints

Approved by Raydot in parent instruction: independent WI scope plus RED → A/B → CLI/MCP → required verification, subject to normal Runtime entry. No duplicate Ray policy review, fake confirm_review, shared Runtime adoption, merge, close or cleanup. Main base 681df30e2fe8c869b6a41498822818c017dc4fb6. Parent retains final technical review.

Pre-flight shared interfaces: A produces only request generation admission; B consumes existing raw decisions and must preserve original state/outcome_state/blockers. Neither authorizes verification, material acceptance or human confirmation. Tests consume exact current Contract/journal/invalidation and observe both CLI/MCP.

Ruling: before-edit admission uses the expressly approved supported native checkpoint after a raw yellow/nonblocking preflight with no review request; coarse status marks any unconfirmed yellow as needs-human — actual checkpoint governance checks decided and returned checkpointed, no validator was bypassed. Cost if wrong: lifecycle admission would need further Raydot interpretation; no tests or Rust changes were made before this native receipt.

Ruling: initial unsupported middle-of-filename boundary globs narrowed to supported directory prefixes and canonical names before preflight/checkpoint — native input validation only, no policy/scope expansion. Cost if wrong: future generated receipt paths could require a separately reviewed boundary adjustment.

Task 1: normal fetch, own branch/worktree, native start/boundary/declaration/preflight/checkpoint complete; original WI-1085 clean and preserved. Native checkpointCount=1. Current source-document material projection remains pending commit and fresh query.
Task 2: RED fixtures and product implementation NOT STARTED.
Task 3: verification and final technical review NOT STARTED.

Task 1: committed source projection 8aeb8e17ac71290a980dff0a1aaa736320ed6f54 pushed and read back. Native post-commit status admission=allowed, humanDecisionRequired=false, safeActions includes run_verification; digest sha256:df05f13cdac7b67a6577a6598946f0abbfafc09c8151a2fb4d72fc6d2be0b161. Initial material projection gap resolved by committing the new WI's own truthful documentation; no old WI parity or policy was changed.

Task 2: first RED targets strict receipt binding before any A/B implementation: CLI and MCP must reject current Contract plus retained pre-amendment Summary preflight fields after native sensitive amendment. Fixture actor is explicitly isolated automated test data; real WI decisionEvidence remains absent. Cargo command uses one job/thread, isolated target, debug info disabled to bound disk, 900-second timeout.

Task 2: RED observed: CLI and MCP both accepted new Contract plus retained pre-amendment preflight fields; 0 passed / 2 failed, exit101, 68.424s, no timeout. Failures are exact rejection assertions; production A/B and validator code unchanged.
Ruling: stop before product changes and send concrete RED plus one-file governance_controls.rs scope extension to Raydot — mandated by approved v2 and parent instruction. Cost if wrong: implementation waits for technical scope/admission review; receipt integrity is not weakened. Prepared typed amendment is not applied and no new human policy approval is requested.
