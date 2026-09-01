---
author: AI Cockpit maintainers
title: "WI-466 — release v0.2.54 and published-adopter acceptance"
workItemId: WI-466-release-v0-2-54
description: "Publish the v0.2.54 Runtime from the reviewed main branch and validate the public binary in an isolated adopter flow."
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: authorized
lastVerifiedBy: WI-466-release-v0-2-54
---

# WI-466 — release v0.2.54 and published-adopter acceptance

## Intent

Publish the patch that contains the closed Work Item documentation-promotion
correction, then prove that the public artifact can initialize and govern an
isolated adopter repository without source or workspace fallback.

## Scope

- Advance the workspace package identity and current release/install guidance
  to `v0.2.54` in all three language routes.
- Run the reviewed release workflow from an annotated tag on synchronized
  `main`, retaining manifest, checksum, SBOM, provenance, and tag evidence.
- Install the published artifact and run public adopter and N-1 acceptance with
  isolated HOME, XDG_CONFIG_HOME, TMPDIR, CARGO_HOME, and adopter repository.
- Preserve `first-adopter-smoke=not_ready`, runtime identity, repository
  identity, evidence reuse, lifecycle receipts, and cleanup proof.

## Out of scope

Reference-source checkout, object repositories, global Agent/MCP configuration,
Homebrew tap mutation, source fallback, Runtime architecture redesign, and
unrelated reference-parity batches.

## Acceptance criteria

1. Workspace package versions and release documentation advance exactly to
   `v0.2.54` without rewriting reserved release history.
2. The reviewed release workflow binds annotated tag, source commit, manifests,
   `SHA256SUMS`, SBOM, provenance, and public artifact identities.
3. Local strict, version, workflow, documentation, and workspace tests pass;
   no source fallback is used.
4. After merge, the published `v0.2.54` binary is downloaded and checksum-
   verified by the post-release adopter acceptance harness, with Runtime
   identity and cleanup receipts.
5. The Runtime repository remains healthy and `ready_on_base` after closure.

## Evidence and verification

The terminal record binds the release tag and public artifacts to the reviewed
source commit. Adopter evidence must retain `runtime.json`, repository and
Work Item identities, lifecycle receipts, evidence-reuse results, isolation
manifests, and cleanup state. The verification command is:

```text
cargo test --locked --workspace
```

The public release and N-1 acceptance scripts are post-release evidence; a
failure never rewrites Release truth.

## Boundary

`v0.2.54` is a same-schema patch. Runtime upgrades remain separate from
repository attachment, and release publication does not attach or mutate an
adopter repository.
