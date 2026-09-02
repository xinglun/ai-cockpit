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

The current immutable release baseline is `v0.2.59` after publication; before
its provider Release exists, `v0.2.58` remains the installable public baseline.
The reserved `v0.2.51` tag is an immutable failed publication attempt
(workflow run `33417057474`), a lightweight tag with no provider Release; it is
never reused. The failed `v0.2.49` tag
is retained as immutable pre-publication history after workflow run
`33379366308`; it has no public Release and is not an installation baseline.
The failed `v0.2.35` tag
is retained as immutable publication history after workflow run `33162800569`
and has no public Release. The earlier failed `v0.2.34` tag remains preserved
after workflow run `33155382717`. The failed staged `v0.2.32`
tag is retained as immutable publication history after the WI-299 finalization
binding defect and has no public Release. The immutable `v0.2.30`
tag remains failed publication history after the clean-batch route defect and
is not an installation baseline.
The `v0.2.56` tag is also immutable failed publication history: source quality
failed before a provider Release existed, so it is never reused or treated as
an installation baseline.
The unpublished `v0.2.36` tag remains immutable staged-acceptance failure
history and is not an installation baseline.
The prior public `v0.2.58` Release remains retained as historical evidence and
is superseded by this baseline; `v0.2.55`, `v0.2.53`, and `v0.2.52` remain earlier historical evidence.
The failed `v0.2.49` tag remains immutable
pre-publication history and has no public Release.
The failed `v0.2.46` tag (`33330269507`) remains immutable publication history;
its public Release was never created because the closed-Work-Item documentation
promotion was missing before tagging, and it is not an installation baseline.

## Purpose

This page answers: **what is trusted during release, how can a person install
the runtime, and where does Homebrew stop?**

Publication starts only from a reviewed, synchronized default branch by
pushing an annotated tag. The provider Release is created by the workflow after
all source, artifact, and staged-acceptance gates pass; maintainers must not
pre-create it with `gh release create`. A lightweight tag or a tag whose peeled
commit is not the reviewed source commit is rejected, and a failed semantic tag
is permanently reserved.

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

## Upgrade boundaries

A Runtime-only upgrade replaces the shared executable and leaves every
repository's `.ai/` bytes, Contract, evidence, Work Item, and knowledge state
unchanged. A Repository migration is different: it is an explicit, reviewed,
versioned operation selected by the new Runtime when compatibility reports
`MIGRATION_REQUIRED`. The migration receipt binds the before/after repository
digests and the Runtime version/digest; historical evidence is not rewritten.

The N-1 acceptance harness proves this boundary with old and new public
archives. It is a post-release artifact, never a source-build fallback or a
replacement for Release truth.

Release tags invoke the harness only after publication and its handoff have
completed; the workflow resolves the preceding published Release and uploads
the receipt independently. Manual dispatch requires explicit public
`from_tag` and `to_tag` inputs and never publishes. When no preceding Release
exists, the workflow records a checksummed `not_applicable` result.
Same-schema patch upgrades still execute the harness and record
`migrationState: not_required`; only a schema-changing pair enters the
approval-gated migration branch.

## Trust boundaries

- `cockpit-release` and the release workflow own the local release contract,
  deterministic manifest, Formula projection, hosted checks, and published
  Release identity.
- The current immutable public baseline is `v0.2.59`; the public adopter
  acceptance and N-1 upgrade acceptance are post-release evidence. An external Homebrew tap is a separate provider surface
  and is not implied by this repository.
- The reserved `v0.2.24` tag and immutable `v0.2.25` tag are retained as failed
  pre-publication history; neither is treated as a public Release or reused.
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
