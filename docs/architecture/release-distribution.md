---
author: AI Cockpit maintainers
title: "Release Distribution Architecture"
description: "How a verified Rust build becomes an installable AI Cockpit runtime without confusing installation with attachment."
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - release_distribution
keywords: [ai-cockpit, release, homebrew, distribution, provenance]
---

# Release Distribution Architecture

## Purpose

This page answers: **what is trusted during release, how can a person install
the runtime, and where does Homebrew stop?**

## Audience

Read it before installing AI Cockpit or reviewing the release pipeline. It is
written for adopters first, with the identity checks called out for maintainers.

## Outcome

You will know which artifact is the source of truth, how the five target builds
are bound together, what the tap handoff may do, and why installation never
silently attaches a repository.

## Release and installation flow

```text
source commit + immutable tag
            │
            ▼
source quality + policy gates
            │
            ▼
five target builds (archive + SBOM)
            │
            ▼
canonical manifest + SHA256SUMS
            │
            ▼
artifact smoke tests + provenance attestation
            │
            ▼
        GitHub Release
       ┌────┼───────────────┬─────────────────┐
       ▼    ▼               ▼                 ▼
 Homebrew  verified       Cargo Git        manual archive
 Formula   archive        fallback          install
       │    │               │                 │
       └────┴───────────────┴─────────────────┘
                         ▼
                   `ai-cockpit`
                         │ explicit attach
                         ▼
       target repository + `.ai/cockpit.toml` + `.ai/project.json`

homebrew-handoff.json ──► external tap review (when a maintained tap exists)
                          (outside this repository's Runtime authority)
```

The release manifest binds version, tag, commit, target, runner image, archive,
SBOM, bytes, digests, and provenance subject. `SHA256SUMS` covers exactly the
archive and SBOM files named by the manifest. A provider Release or uploaded
asset alone is not installation evidence.

## What an adopter does

1. Install from the published Homebrew Formula, or download the matching
   archive from the immutable Release.
2. Verify the version, SHA-256 digest, and provider attestation.
3. Run `ai-cockpit attach --repo /path/to/repository` only after reviewing the
   target repository and its Work Item. Attachment is explicit and is the step
   that may create or update `.ai/`.
4. Start the CLI or MCP adapter against the attached repository.

An unattached release-build checkout may remain without `.ai`; this self-governed
checkout intentionally owns its repository-local `.ai/`. `cockpit.toml` remains
TOML under `.ai/`; distribution does not migrate it to JSON.

## Trust boundaries

- `cockpit-release` and the release workflow own the local release contract,
  deterministic manifest, Formula projection, hosted checks, and published
  Release identity.
- The current immutable public baseline is `v0.1.1`; WI-40 records its public
  adopter acceptance. An external Homebrew tap is a separate provider surface
  and is not implied by this repository.
- The tap receives a reviewed Formula projection; it does not rebuild binaries.
- Homebrew is a delivery path, not a governance authority. Repository facts and
  human decisions still come from the attached repository and its Work Items.

## Stop conditions

Stop when the tag, workspace version, binary version, commit, manifest, digest,
SBOM, provenance subject, or provider Release identity disagree. Stop when a
handoff is expired, points at another commit, asks for a different destination,
or attempts a direct default-branch mutation. Stop when installation is being
presented as proof that a repository has been attached.

## Next steps

1. [Release and distribution](../release/distribution.md) — adopter commands.
2. [Architecture](../architecture.md) — runtime and evidence ownership.
3. [Reference source parity](../reference/reference-parity.md) — explicit
   differences from the reference template.

## Technical depth

The Rust `cockpit-release` package performs strict manifest, archive, Formula,
and handoff validation. GitHub Actions builds five retained targets, separates
source, verification, attestation, publication, and handoff permissions, and
keeps the external tap mutation outside the default repository token.
