---
author: AI Cockpit maintainers
title: "WI-117 Release adopter toolchain isolation"
description: "Bind the N-1 acceptance harness to an existing Rust toolchain without implicit downloads."
audience:
  - maintainer
  - release-engineer
status: implemented
authority: canonical
lastVerifiedBy: release-adopter-toolchain-regression
capabilityClaims:
  - bounded_release_acceptance
  - isolated_toolchain_identity
---

# WI-117: Release adopter toolchain isolation

## Goal

Make post-release adopter and N-1 acceptance deterministic when the harness
uses an isolated HOME, TMPDIR, and CARGO_HOME.

## Scope

The N-1 harness resolves the host-provided Rustup home and active toolchain,
passes them explicitly into the isolated fixture commands, and refuses an
implicit network toolchain download when either identity is unavailable. The
static regression checks the success and fail-closed paths. Runtime protocol
semantics and global Rust installation remain out of scope.

## Acceptance

- `RUSTUP_HOME` falls back to `rustup show home` when the environment variable
  is absent.
- `RUSTUP_TOOLCHAIN` is resolved from the active toolchain and passed to every
  isolated Cargo/Runtime invocation.
- Missing toolchain identity fails closed before creating an unbounded fixture.
- Cleanup evidence remains separate from acceptance truth and removes only the
  validated temporary run root.
- English, Chinese, and Japanese release documentation describe this boundary.

## Verification

```text
bash tests/release/adopter_upgrade_acceptance_test.sh
bash tests/docs/documentation_acceptance.sh
git diff --check
```

## Outcome

Status: **Implemented; toolchain identity and bounded cleanup are explicit.**
