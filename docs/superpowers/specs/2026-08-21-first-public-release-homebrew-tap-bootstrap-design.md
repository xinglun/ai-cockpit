# First Public Release and Homebrew Tap Bootstrap Design

**Status:** Opened for WI-35; no external write is authorized by this document.

## Goal

Publish the first immutable `v0.1.0` release from the locally accepted WI-34
candidate, then hand the exact Formula to a protected Homebrew tap through a
reviewable pull request. A user must be able to verify the source identity,
artifacts, checksums, SBOMs, provenance, provider Release, and installation
result as one connected chain.

## Reader-facing contract

The release is a delivery path for the Rust `ai-cockpit` binary. Installing it
does not attach a repository, create `.ai/`, or change `cockpit.toml` to JSON.
Repository attachment remains an explicit `ai-cockpit attach --repo <path>`
operation after installation and review.

## Identity chain

```text
approved commit
  └─ new immutable v0.1.0 tag
      └─ source-quality and policy gates
          └─ five target builds + SBOMs
              └─ canonical manifest + SHA256SUMS
                  └─ archive/smoke/provenance gates
                      └─ provider Release
                          └─ post-publication handoff
                              └─ protected tap PR
```

Every link carries the version, tag, commit, target, digest, and the identity
of the operation that produced it. The Homebrew Formula is a projection of the
canonical manifest and never a second build authority.

## Workflow boundaries

- `build` creates target archives and SBOMs with read-only repository access.
- `source_quality` runs format, Clippy, tests, metadata, and workflow policy.
- `aggregate` creates and validates the one canonical candidate.
- `release_policy` rejects moved tags, reused provider Releases, version drift,
  and commits outside the approved default branch.
- `verify` and the three smoke jobs prove artifact structure and runtime use.
- `attest` attests the final candidate bundle after every preceding gate.
- `publish` is tag-push-only and uploads an explicit asset allowlist.
- `publish_handoff` runs only after publication, binds the provider Release ID,
  and attests the handoff consumed by the external tap verifier.

Manual dispatch may build and verify a candidate, but it cannot publish a
provider Release merely because a tag ref was selected.

## Handoff contract

`homebrew-handoff.json` contains the exact workflow ref and run, tag, commit,
provider Release ID, manifest digest, Formula digest, issue/expiry timestamps,
and the requested external destination/action. The tap verifier must reject a
missing, expired, re-used, or differently bound handoff. The tap default branch
is never mutated directly by this workflow.

## Failure and recovery

Any identity mismatch, missing asset, failed smoke, failed attestation, unknown
provider API response, or tap-policy mismatch stops publication. The next action
is a corrective Work Item or a new candidate; an existing tag or Release is not
overwritten. Hosted evidence is recorded in WI-35 and is not backfilled from
local tests.

## Out of scope

This design does not install V1 or `.ai/` into the development checkout, alter
Rust governance semantics, publish to crates.io, claim Linuxbrew support, or
authorize credentials, tags, Releases, tap creation, pull requests, or public
installation testing.
