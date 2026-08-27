---
author: AI Cockpit maintainers
title: CI Runtime verification shadow
description: Typed repository quality routing plus an immutable public Runtime execution shadow.
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-224-ci-reference-parity
---

# CI Runtime verification shadow

WI-224 makes the repository CI route explicit. `quality_route.py` selects
`light`, `standard`, or `strict` from the changed paths, Contract risk, and
workflow stage. Unknown paths, release-owned paths, high risk, merge, and
release stages escalate to `strict`. The typed route receipt binds the Git base
and head, changed paths, Contract path and digest, manifest byte digest,
selection reasons, and ordered gate IDs. `run_repository_gates.py` recomputes
that receipt from repository facts and executes only commands stored in the
canonical manifest; it has no arbitrary command override.

Runtime shadow is Contract-aware. A `standard` or `strict` pull request runs
and uploads the shadow only when the initial route resolved one active
Contract. An archived, post-finish pull request with no active Contract still
runs the ordinary repository gates, but skips this execution-only shadow because
the immutable Runtime cannot produce Work-Item verification evidence without a
current Contract. This explicit skip does not weaken the selected gates or turn
missing evidence into a pass.

The profiles are cumulative. `light` runs documentation and governance-policy
regressions, `standard` adds the Cargo format/Clippy/package gates plus the
immutable Runtime shadow and source conformance, and `strict` adds release,
workflow, performance, adopter, and source-archive gates. Pull requests use the
path/risk route; merge pushes have a strict stage floor. Release source quality
always requests `strict` and uploads both the route receipt and gate report.

CI uses two bounded route plans. The initial receipt decides whether the
Runtime shadow is required; `light` skips that shadow. A `standard` or `strict`
route runs the shadow and then recomputes the final receipt from the same
immutable Git base/head plus any repository-local Runtime writes, including
`.ai/evidence/reuse/**`. The gate runner consumes only this final receipt, while
both receipts remain available for diagnosis. Workspace package coverage is
required for the final non-light profile and uploaded only when its regular
receipt file exists; a valid `light` route neither requires nor uploads it.

For `standard` and `strict`, the independent execution shadow downloads the
public immutable `v0.2.28` Runtime, verifies the platform archive and binary
digests, and runs canonical repository-profile verification. Its receipt binds
the tag, version, archive digest, binary digest, platform, download source, and
Runtime result. It rejects source builds, workspace binaries, arbitrary
`--command` substitution, unpinned artifacts, digest mismatch, and malformed
output.

This is a repository CI/release policy. It does not claim Runtime-global T0–T3
route selection, affected-graph completeness, cross-Work-Item physical
execution, or generic CLI `verify --command` semantics; those Runtime changes
remain deferred because WI-224 does not authorize `crates/**`. The shadow is an
execution identity check, not a substitute for the selected manifest gates or
provider/enterprise assurance.
