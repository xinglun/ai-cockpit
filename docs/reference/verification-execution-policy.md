---
author: AI Cockpit maintainers
title: Verification execution cache policy
description: Keep Cargo verification caches bounded and reusable across Work Items.
audience: [adopter, contributor, maintainer]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-879-verification-target-policy
---

# Verification execution cache policy

When the Runtime launches a Cargo verification command, it resolves the
execution environment before starting the child process:

- `CARGO_INCREMENTAL=0` disables incremental artifacts that have little value
  for bounded verification runs.
- `CARGO_TARGET_DIR` is set to `$HOME/.cache/ai-cockpit-verify-target` (or the
  platform profile directory equivalent) so repeated Work Items reuse compiled
  dependencies instead of creating one `target` tree per checkout.
- Non-Cargo commands keep their declared environment unchanged.

The policy is part of Runtime execution, not a shell convention. The effective
environment is included in the verification observation identity, so changing
the policy or toolchain cannot silently reuse an unrelated receipt. Planning
still completes before any child process is started.

After a verification session, first confirm that no `cargo` or `rustc` process
is running. A repository-local incremental directory may then be removed with
`rm -rf target/debug/incremental`; this does not remove the shared target cache
or its reusable dependencies. Do not use `cargo clean` as routine cleanup.

This policy controls cache placement and does not grant verification,
authorization, merge, release, or human-approval status.
