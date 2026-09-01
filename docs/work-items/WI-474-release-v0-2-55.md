---
author: AI Cockpit maintainers
title: "WI-474 — release v0.2.55 and public adopter acceptance"
description: "Publish the reviewed Runtime patch and validate the immutable public binary without changing adopter repositories."
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: authorized
lastVerifiedBy: WI-474-release-v0-2-55
workItemId: WI-474-release-v0-2-55
---

# WI-474 — release v0.2.55 and public adopter acceptance

## Intent

Publish the next reviewed Runtime patch, then prove that the immutable public
binary can be installed and used to govern an isolated adopter. This release
continues the mainline reference comparison; it does not alter the reference
source or an adopter repository.

## Scope

- Advance the workspace package identity and current tri-language release and
  versioning guidance to `v0.2.55`, preserving historical release facts.
- Register this Work Item in the three reference-parity ledgers before archive.
- Merge a reviewed PR, publish an annotated tag, and retain manifest,
  checksum, SBOM, provenance, and artifact identity evidence.
- Run public adopter and N-1 acceptance only from downloaded Release artifacts
  in isolated roots, including evidence reuse and temporary-root cleanup.
- Install or upgrade the published binary on this repository and verify its
  repository, Runtime, Agent, and readiness state.

## Out of scope

The local reference source, `/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator`,
other adopter repositories, global Agent/MCP configuration, Homebrew tap
mutation, source fallback, and unrelated reference-parity or Runtime
architecture changes are out of scope.

## Acceptance criteria

1. Workspace package versions, lockfile entries, and required tri-language
   release/versioning documents identify `v0.2.55` without rewriting history.
2. A reviewed PR passes hosted checks before merge; the annotated `v0.2.55`
   tag points exactly to the synchronized reviewed main commit.
3. The public Release exposes the expected archives, checksums, SBOM,
   provenance metadata, and identity-bound release manifest.
4. Public adopter and N-1 acceptance use only immutable public artifacts,
   preserve `first-adopter-smoke=not_ready`, bind repository/Runtime identity,
   prove forbidden-root isolation, and prove temporary-root cleanup on both
   success and failure paths.
5. The published binary is installed on this repository and
   `inspect`/`status`/`doctor`/`agent doctor` confirm attached, healthy,
   isolated, and `ready_on_base` state.
6. The Work Item reaches a visible human Outcome, archive/finalization/close,
   promoted documentation, and exact branch/worktree cleanup.

## Verification

The source verification command is:

```text
cargo test --locked --workspace
```

Release publication and public adopter acceptance are post-release evidence;
they do not rewrite Release truth if they fail.

## Boundary

Runtime upgrades replace the shared executable while repository Protocol state
remains local. Publication does not attach or mutate an adopter repository.
