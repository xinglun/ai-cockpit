---
author: AI Cockpit maintainers
title: "CI and Release Evidence"
description: "Provider-derived CI and public Release evidence with explicit ownership."
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: WI-331-checks-release-evidence
keywords: [ai-cockpit, ci, release, evidence]
---

# CI and Release Evidence

CI and Release records are delegated evidence. Their authority comes from the
hosted provider and the exact published artifact, not from a pull-request body,
Agent message, or local assertion that something “passed”. The Rust Runtime can
bind and validate those records, but it does not impersonate GitHub Actions or
an enterprise approval system.

## CI evidence

The versioned `tests/ci/repository_gate_manifest.json` and the CI route bind the
repository, Contract, base revision, head revision, selected profile, ordered
gate IDs, and route/manifest digests. The final gate report records each
required gate and its result. A hosted adapter should additionally retain the
provider workflow run, job names, job conclusions, and exact head SHA.

Required jobs are an explicit set. A skipped or failed job remains in the
record; it is never omitted to make the aggregate look green. The aggregate
conclusion must agree with every job result and failure reason. A PR body or
human prose cannot replace a provider run, and a local fixture cannot be
promoted to hosted assurance.

The profile is policy-selected and cumulative: `light`, `standard`, and
`strict` are verification coverage, not assurance levels. A merge or release
stage has a strict floor. Unknown paths and release-owned files fail closed to
the strict route. The Rust Contract gate remains the authority for the
repository-bound decision; the existing script runner is a bounded execution
shadow during convergence.

## Release evidence

The release workflow binds a version, tag, source commit, Cargo.lock digest,
target archive, executable member, checksum manifest, SBOM, and provenance.
Each target must have the expected archive layout and a checksum that is
recomputed from the bytes actually published. SBOM and attestation subjects
must refer to the same source and artifact identity. A tag or uploaded file by
itself is not a stable public Release.

Release evidence has separate states:

| State | Meaning | Authorization boundary |
| --- | --- | --- |
| `candidate` | A staged source/artifact record before publication. | May support review; does not prove a public Release. |
| `verified` | Provider evidence for an exact source commit with required jobs and assets passing. | May support the publication step; still not a published Release. |
| `published` | Verified evidence attached to the exact public Release and asset set. | Public publication fact, not enterprise certification. |
| `failed` | A provider or artifact check failed and includes its reasons. | Never authorizes `verified` or `published`. |

The post-release adopter harness adds a separate acceptance receipt. It binds
the downloaded immutable tag/artifact, binary and archive digests, isolated
repository identity, lifecycle evidence, and cleanup/isolation manifests. A
successful adopter receipt is evidence that this binary governed that adopter;
it is not proof that every technology stack or every enterprise environment is
covered.

## Ownership and failure

Local Runtime and manifest checks are repository evidence. Hosted run/job
results, merge observations, signing, SBOM publication, attestations, branch
protection, and enterprise approvals remain external or provider-owned
evidence. AI Cockpit records their identity, origin, assurance, collection time,
digest, validity, and raw reference when supplied; it never fabricates a
provider result.

Missing jobs, skipped/failed jobs hidden from the aggregate, head/base mismatch,
wrong artifact or SBOM digest, duplicate or missing checksum entries, malformed
JSON, or a release state without provider-bound evidence fail closed. Preserve
the failure receipt and source identity; do not rewrite an already published
Release as unpublished and do not reuse a failed receipt for a later version.

The same boundary applies to an adopter project: the shared Runtime is outside
the project, repository state is isolated under `.ai/`, and every command uses
an explicit `--repo <path>`.
