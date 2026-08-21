# Release Adopter Acceptance Baseline Design

## Goal

Establish a reproducible post-release adopter acceptance baseline using only
immutable public Release artifacts. The baseline answers whether one public
GitHub Release binary can create and govern a completely fresh adopter
repository without source-tree or global-agent fallback.

## Boundary

The first implementation is a versioned shell script,
`tests/release/adopter_acceptance.sh`, plus a post-publication GitHub Actions
job. No `ai-cockpit acceptance` Runtime command is added in this Work Item.
The script is an acceptance harness, not a governance capability exposed to
adopter repositories.

The script must never execute `cargo build`, `cargo run`, a workspace binary,
or a local `target` binary to obtain AI Cockpit. It downloads the requested
archive from the public Release, verifies its manifest checksum, extracts one
pinned binary, and invokes that absolute path for every Runtime operation.
Cargo is allowed only inside the temporary adopter for its ordinary test
command, which the pinned Runtime verifies.

## Inputs and outputs

```text
tests/release/adopter_acceptance.sh \
  --repository OWNER/REPOSITORY \
  --tag vX.Y.Z \
  --target TARGET \
  --output DIRECTORY
```

`--repository`, `--tag`, `--target`, and `--output` are explicit in CI. The
script also accepts the current GitHub repository, tag, and runner target from
the environment when invoked by the release workflow. Missing or ambiguous
values fail closed.

The output directory contains raw JSON evidence and two binding files:

```text
release-adopter-acceptance/
├── acceptance.json
├── SHA256SUMS
├── release.json
├── runtime.json
├── repository.json
├── attach.json
├── profile-confirm.json
├── agent-list.json
├── agent-install.json
├── agent-doctor.json
├── verify-first.json
├── verify-reuse.json
├── isolation.json
└── work-items/
    ├── first-adopter-smoke.contract.json
    ├── lifecycle.start.json
    ├── lifecycle.checkpoint.json
    ├── lifecycle.preflight.json
    ├── lifecycle.verify.json
    ├── lifecycle.finish.json
    ├── lifecycle.archive.json
    ├── lifecycle.close.json
    ├── lifecycle.contract.json
    ├── lifecycle.outcome.json
    └── lifecycle.evidence.json
```

`SHA256SUMS` covers every evidence file except itself. `acceptance.json` is the
summary record and includes `releasePublished`, `adopterAcceptance`, step
states, repository identity, runtime identity, timestamps, and failure reasons.
When a post-publication step fails, `releasePublished` remains `true` and only
`adopterAcceptance` becomes `failed`; the script never edits or reclassifies the
already-published Release.

## Runtime identity contract

`runtime.json` records `tag`, `version`, `archiveDigest`, `binaryDigest`,
`platform`, `archive`, `downloadSource`, `releaseUrl`, and
`releasePublished: true`. The script compares the Runtime `runtimeVersion` and
`runtimeDigest` in `doctor`, `inspect`, and Work Item verification evidence to
the downloaded version and binary digest. A mismatch fails the acceptance.

## Acceptance flow

1. Query the public Release API and require the exact immutable tag to be
   published, non-draft, and non-prerelease.
2. Download only the matching archive, `release-manifest.json`, and
   `SHA256SUMS`; verify the exact archive digest and manifest version/tag.
3. Create isolated `HOME`, `XDG_CONFIG_HOME`, `TMPDIR`, `CARGO_HOME`, and a
   fresh temporary Git adopter. Commit its initial Cargo scaffold before
   attaching it.
4. Run `attach`, `profile confirm`, create one unique `AGENTS.md`, and run
   `agent list`, `agent install --provider auto`, and `agent doctor --json`.
5. Create `first-adopter-smoke` with `work-item new`; assert its contract is
   `not_ready` and its human-owned `intent`, `scope`, `acceptanceCriteria`, and
   `authority` remain empty or `unknown`.
6. Run a separate lifecycle Work Item through `start`, `checkpoint`,
   `preflight`, `verify --work-item`, `finish`, `archive`, and
   `close --human-decision approved`, preserving every receipt.
7. Commit the adopter state, run the pinned Runtime twice in an identical
   sanitized environment, and require the first run to execute and the second
   run to reuse at least one receipt with zero spawned processes.
8. Prove the source checkout has no `.ai/` and the isolated HOME/XDG trees have
   identical before/after manifests. Emit `acceptance.json` and `SHA256SUMS`.

## CI boundary

The `adopter_acceptance` job runs only for a tag push, after `publish` and
`publish_handoff`. It downloads the public Release by tag rather than the
candidate artifact. The job uploads the acceptance directory even when the
script fails, so a published Release with a failed adopter acceptance remains
auditable and cannot authorize a later version.

## Out of scope

This Work Item does not add a Runtime CLI command, change the repository
protocol, install a provider globally, test a second language ecosystem, or
make post-release acceptance a pre-publication gate. A Node/npm adopter is a
separate future Work Item.
