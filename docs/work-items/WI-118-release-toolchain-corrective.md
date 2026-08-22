# WI-118 — Release toolchain and cleanup fail-closed correction

## Objective

Make public adopter and N-1 release acceptance deterministic and fail closed.
This Work Item supersedes the implementation review findings from WI-117; it
does not rewrite WI-117's archived evidence.

## Scope

- Bind the host `RUSTUP_HOME` and active toolchain explicitly in both release
  harnesses before entering isolated roots.
- Make a cleanup failure return non-zero and mark `adopterAcceptance` failed,
  while preserving `releasePublished: true`.
- Add static regressions and synchronize the release documentation.

## Out of scope

Runtime protocol semantics, global Rust installation, and mutation of an
already-published Release.

## Acceptance

1. Public and N-1 harnesses reject missing toolchain identity and never rely on
   implicit rustup downloads.
2. Cleanup failures are visible in `cleanup.json` and `acceptance.json`, fail
   the process, and cannot leave a passing receipt.
3. Release publication truth remains immutable when post-release cleanup fails.
4. Static harness tests and all three release documentation languages pass.
