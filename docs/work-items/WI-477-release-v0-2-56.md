---
author: AI Cockpit maintainers
title: "WI-477 — release v0.2.56 and public adopter acceptance"
description: "Publish the reviewed Runtime patch and validate the immutable public binary without changing adopter repositories."
audience:
  - adopter
  - maintainer
  - reviewer
status: implementation_active
authority: canonical
lastVerifiedBy: WI-477-release-v0-2-56
workItemId: WI-477-release-v0-2-56
---

# WI-477 — release v0.2.56 and public adopter acceptance

## Intent

Publish the next reviewed Runtime patch, prove that its immutable public
binary can govern an isolated adopter, install it on this repository, and then
return to the local reference-source comparison. This Work Item does not
modify the reference source or any adopter repository.

## Scope

- Align workspace package identity and the current tri-language release and
  versioning guidance to `v0.2.56`, preserving historical release facts.
- Register this Work Item in the tri-language parity ledgers before archive.
- Merge a reviewed PR, publish an annotated tag, and retain manifest,
  checksum, SBOM, provenance, and artifact identity evidence.
- Run public adopter and N-1 acceptance only from downloaded Release artifacts
  in isolated roots, including evidence reuse and temporary-root cleanup.
- Install or upgrade the published binary on this repository and verify its
  repository, Runtime, Agent, and readiness state.

## Out of scope

The local reference source, `/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator`,
other adopter repositories, global Agent/MCP configuration, Homebrew tap
mutation, source fallback, and unrelated Runtime architecture changes.

## Acceptance criteria

1. Workspace versions, lockfile entries, and required tri-language release
   documents identify `v0.2.56` without rewriting history.
2. A reviewed PR passes hosted checks before merge, and the annotated
   `v0.2.56` tag points exactly to the synchronized reviewed main commit.
3. The public Release exposes expected archives, checksums, SBOM/provenance,
   and identity-bound release manifest.
4. Public adopter and N-1 acceptance use only immutable public artifacts,
   preserve `first-adopter-smoke=not_ready`, bind identities and digests,
   prove isolation, and prove cleanup on success and failure.
5. The published binary is installed on this repository and
   `inspect`/`status`/`doctor`/`agent doctor` confirm healthy `ready_on_base`.
6. The Work Item reaches visible human Outcome, archive/finalization/close,
   promoted documentation, and exact branch/worktree cleanup.

## Verification

```text
cargo test --locked --workspace
```

Release publication and public acceptance are post-release evidence. A
failure records the failure without rewriting existing Release truth.

## Boundary

Runtime upgrades replace the shared executable while repository Protocol state
remains local. Publication never implicitly attaches or mutates a repository.
