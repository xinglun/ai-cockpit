# SDD ledger — plan: ../admission-20261016/approved-proposal-v2.zh-CN.md

## Binding and constraints

Approved by Raydot in parent instruction: independent WI scope plus RED → A/B → CLI/MCP → required verification, subject to normal Runtime entry. No duplicate Ray policy review, fake confirm_review, shared Runtime adoption, merge, close or cleanup. Main base 681df30e2fe8c869b6a41498822818c017dc4fb6. Parent retains final technical review.

Pre-flight shared interfaces: A produces only request generation admission; B consumes existing raw decisions and must preserve original state/outcome_state/blockers. Neither authorizes verification, material acceptance or human confirmation. Tests consume exact current Contract/journal/invalidation and observe both CLI/MCP.

Ruling: before-edit admission uses the expressly approved supported native checkpoint after a raw yellow/nonblocking preflight with no review request; coarse status marks any unconfirmed yellow as needs-human — actual checkpoint governance checks decided and returned checkpointed, no validator was bypassed. Cost if wrong: lifecycle admission would need further Raydot interpretation; no tests or Rust changes were made before this native receipt.

Ruling: initial unsupported middle-of-filename boundary globs narrowed to supported directory prefixes and canonical names before preflight/checkpoint — native input validation only, no policy/scope expansion. Cost if wrong: future generated receipt paths could require a separately reviewed boundary adjustment.

Task 1: normal fetch, own branch/worktree, native start/boundary/declaration/preflight/checkpoint complete; original WI-1085 clean and preserved. Native checkpointCount=1. Current source-document material projection remains pending commit and fresh query.
Task 2: RED fixtures and product implementation NOT STARTED.
Task 3: verification and final technical review NOT STARTED.
