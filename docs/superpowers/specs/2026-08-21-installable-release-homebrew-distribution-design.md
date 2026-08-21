# Installable Release and Homebrew Distribution Design

## Context

WI-24 defines a five-target GitHub Actions build with checksums, SBOMs,
provenance attestations, optional checksum signatures, and tag-triggered GitHub
Release publication. It does not define a user installation contract. The
repository has no semantic tags or provider Releases, and
`xinglun/homebrew-tap` does not exist as of 2026-08-21. The binary package is
named `cockpit-cli`, while the installed executable is `ai-cockpit`.

The user approved a GitHub-Release-first distribution with Homebrew as the
primary installation path. `cockpit.toml` remains the repository configuration
format; distribution work does not migrate it to JSON or install V1 into this
development repository.

## Approaches Considered

### Selected: immutable GitHub assets plus an upstream Homebrew tap

GitHub Release assets are the artifact source of truth. A deterministic
generator projects the two macOS archives and their SHA-256 values into
`Formula/ai-cockpit.rb` in `xinglun/homebrew-tap`. Users install the fully
qualified formula with `brew install xinglun/tap/ai-cockpit`, limiting trust to
that formula.

This keeps build provenance in the source repository, makes installation fast,
and keeps tap history independently reviewable. It does require a separately
protected tap repository and a cross-repository handoff.

### Rejected for the first release: Homebrew core source build

Homebrew core would improve discovery but adds an external acceptance cycle and
would build the Rust workspace from source. It is unsuitable as the first
release gate and can be reconsidered after stable public releases exist.

### Rejected as the primary path: shell bootstrap installer

A `curl | sh` installer reduces repository setup but introduces an executable
network bootstrap and a second update mechanism. Manual archive installation
will be documented, but no remote shell pipeline becomes the recommended path.

Crates.io publication is also deferred. Publishing `cockpit-cli` would require
independent packaging of its path-based workspace dependencies and is not a
prerequisite for a verified Homebrew release. A tag-pinned, locked
`cargo install --git` command remains a developer fallback.

## Work Item Decomposition

The design crosses two independent governance domains and is therefore split:

1. **WI-34 — Installable Release and Homebrew Distribution Readiness** builds
   and verifies the release contract, manifest, Formula generator, smoke tests,
   documentation, and identity-bound external handoff in this repository.
2. **WI-35 — First Public Release and Tap Bootstrap** creates and protects the
   external tap, executes hosted release gates, creates the first immutable tag
   and provider Release, submits/merges the exact Formula update, and records
   real installation receipts. WI-35 requires explicit external-write and
   publication authorization.

WI-34 must be independently useful: it produces a complete, reproducible
release candidate and proves every installation path against staged artifacts.
It does not claim that a public Release or tap exists.

## Release Identity Contract

The first candidate is `v0.1.0` because the workspace version is `0.1.0` and
the remote currently has no tags or Releases. Every candidate must satisfy all
of these exact bindings before publication:

- tag name is `v<workspace-version>`;
- `cargo metadata --locked` reports that version for every workspace package;
- `ai-cockpit --version` reports the same version;
- the tag target, source commit, Cargo.lock digest, and workflow source commit
  are identical;
- every archive name contains one supported Rust target triple;
- one canonical `release-manifest.json` records schema version, product,
  version, tag, commit, archive filename, target, OS, architecture, byte size,
  SHA-256, SBOM filename, and provenance subject;
- one canonical `SHA256SUMS` covers every published archive and SBOM named by
  the manifest.

An existing tag, mismatched version, missing asset, duplicate target, checksum
disagreement, mutable source identity, or unsupported target fails closed.
`workflow_dispatch` may build and verify a candidate but cannot publish a
Release or update a tap.

### Canonical release manifest

`release-manifest.json` is UTF-8 JSON with one trailing LF. A dedicated typed
serializer emits the fields in the order shown below, without insignificant
whitespace. Artifact entries are sorted by `target`; duplicate targets or
filenames and unknown fields are rejected. Digests are exactly 64 lowercase
hexadecimal SHA-256 characters without a prefix. Byte counts are unsigned
integers. The manifest itself is not listed inside itself.

```json
{"schemaVersion":1,"product":"ai-cockpit","package":"cockpit-cli","version":"0.1.0","tag":"v0.1.0","commit":"<40-lowercase-hex>","cargoLockSha256":"<64-lowercase-hex>","artifacts":[{"target":"aarch64-apple-darwin","os":"macos","architecture":"arm64","runnerImage":"macos-15","archive":{"filename":"ai-cockpit-v0.1.0-aarch64-apple-darwin.tar.gz","bytes":123,"sha256":"<64-lowercase-hex>"},"sbom":{"filename":"ai-cockpit-v0.1.0-aarch64-apple-darwin.spdx.json","bytes":123,"sha256":"<64-lowercase-hex>"},"provenanceSubject":"ai-cockpit-v0.1.0-aarch64-apple-darwin.tar.gz"}]}
```

The real manifest contains exactly one entry for each of the five retained
targets. `provenanceSubject` must equal that entry's archive filename, and the
attestation subject digest must equal `archive.sha256`. `SHA256SUMS` is sorted
bytewise by filename and contains exactly the ten archive/SBOM files. Manifest
validation recomputes every size and digest from staged files rather than
trusting producer metadata.

## Artifact Layout

The retained target matrix is:

| Platform | Rust target | Archive |
| --- | --- | --- |
| macOS ARM64 | `aarch64-apple-darwin` | `.tar.gz` |
| macOS Intel | `x86_64-apple-darwin` | `.tar.gz` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | `.tar.gz` |
| Linux x86_64 | `x86_64-unknown-linux-gnu` | `.tar.gz` |
| Windows x86_64 | `x86_64-pc-windows-msvc` | `.zip` |

WI-34 does not infer broad minimum operating-system or ABI support from a Rust
target triple. Its release workflow replaces moving runner aliases with the
following explicit validation floor: macOS 15 Intel, macOS 15 ARM64, Ubuntu
24.04 for both GNU/Linux architectures, and Windows Server 2025 x86_64.
Homebrew installation is additionally exercised on the current macOS 15 ARM64
and Intel images. Each manifest entry records the build runner image, and the
release receipt records the observed kernel and, for GNU/Linux, glibc version.
Older releases, other distributions, and broader ABI compatibility remain
unsupported until separately measured; these are validation environments, not
claims that every newer or older system is compatible.

Each archive contains exactly one executable named `ai-cockpit` or
`ai-cockpit.exe`, plus `LICENSE` and a short archive README. At WI-34 opening the
repository declared MIT in Cargo metadata but lacked the root `LICENSE`; WI-34
now adds the exact approved `Copyright (c) 2026 Ray` notice, and an absent or
unapproved notice fails the release gate. The executable is
not target-suffixed inside the archive. The Formula can therefore install
`bin.install "ai-cockpit"` without filesystem discovery or renaming heuristics.

Linux Homebrew support is deferred until the GNU binary compatibility floor is
measured on supported Linuxbrew environments. Linux users receive verified
archives and the Cargo Git fallback in WI-34; the Formula supports macOS ARM64
and Intel only.

## Formula Projection

The source repository owns a deterministic Formula generator, not a second
hand-maintained Formula. It consumes only a strict `release-manifest.json` and
emits one Ruby file with:

- `desc`, HTTPS `homepage`, `license "MIT"`, and explicit version;
- `on_macos` plus `on_arm`/`on_intel` asset URLs and exact SHA-256 values;
- a fail-closed unsupported-platform guard;
- `def install` that installs the archive's `ai-cockpit` into `bin`;
- `test do` assertions for `ai-cockpit --version` and `ai-cockpit --help`.

The production URL format is
`https://github.com/xinglun/ai-cockpit/releases/download/<tag>/<archive>`.
Generation rejects non-HTTPS production URLs, path traversal, unknown manifest
fields, malformed digests, missing macOS variants, duplicate assets, and a
version/tag mismatch. Repeated generation from identical input is byte-for-byte
stable.

## Workflow and Trust Separation

Release jobs use least privilege per job rather than workflow-wide write
permissions. Build jobs read source; attestation jobs receive only the required
OIDC/attestation permissions; the Release job alone receives `contents: write`.
Third-party actions are pinned to reviewed full commit SHAs.

After a tagged candidate passes source tests, target builds, manifest checks,
archive smoke tests, and attestations, the Release job publishes the exact
merged artifacts. A later Homebrew handoff job consumes the published manifest
and generated Formula; it never rebuilds binaries.

Cross-repository mutation is not performed with the default `GITHUB_TOKEN`.
WI-34 emits `homebrew-handoff.json`, a canonical UTF-8 JSON document serialized
with the same ordering, whitespace, digest, duplicate, and unknown-field rules
as the release manifest. Its normative shape is:

```json
{"schemaVersion":1,"requestId":"<64-lowercase-hex>","issuer":{"repository":"xinglun/ai-cockpit","workflowRef":"xinglun/ai-cockpit/.github/workflows/release.yml@<40-lowercase-hex>","runId":123},"destination":{"repository":"xinglun/homebrew-tap","baseRef":"main","path":"Formula/ai-cockpit.rb"},"release":{"tag":"v0.1.0","commit":"<40-lowercase-hex>","providerReleaseId":123,"manifestSha256":"<64-lowercase-hex>","formulaSha256":"<64-lowercase-hex>"},"authorizedAction":"open_pull_request","issuedAt":"2026-08-21T00:00:00Z","expiresAt":"2026-08-22T00:00:00Z"}
```

`requestId` is the SHA-256 of the same canonical object with `requestId`
omitted. Timestamps are UTC RFC 3339 seconds; expiry must be after issuance and
no more than 24 hours later. WI-35 retrieves the handoff from the named source
run, verifies its GitHub attestation, source workflow ref, exact source commit,
manifest and Formula bytes, destination allowlist, action, and expiry with at
most five minutes of clock skew. A dedicated GitHub App installation or
narrowly scoped protected credential may then open a PR. The deterministic PR
branch includes `requestId`: retrying the same request is idempotent, a consumed
request cannot authorize a different diff, and an expired or conflicting
request requires a newly attested handoff. Direct pushes to the tap default
branch are prohibited.

## Verification Strategy

Local and hosted verification are separate evidence classes:

- Rust unit tests validate manifest parsing, version binding, checksums,
  deterministic Formula generation, invalid inputs, and handoff identity.
- A local release-fixture test packages the current binary and validates exact
  archive membership, executable permissions, checksum coverage, and manifest
  round trips.
- Hosted target jobs extract their own archive and run `--version` and `--help`.
- `macos-15` ARM64 and `macos-15-intel` jobs serve staged archives from an
  ephemeral loopback HTTP server and install a separately typed test-only
  Formula, run `brew test`, verify the linked executable, uninstall it, and
  confirm the link is absent. The fixture generator accepts only
  `http://127.0.0.1:<ephemeral-port>/...`, is not callable by the production
  generator, stamps the Formula as test-only, and cannot emit a handoff. The
  production generator continues to accept only the fixed GitHub HTTPS origin.
  These tests do not require a public Release.
- Windows verifies the ZIP with PowerShell SHA-256 and executes the extracted
  `.exe`; Linux verifies both native target artifacts on matching runners.
- A local temporary Git fixture creates an isolated `v0.1.0` tag. The smoke
  contract is `cargo install --git file://<absolute-fixture-path> --tag v0.1.0
  --locked --root <temporary-root> --bin ai-cockpit cockpit-cli`, followed by
  `<temporary-root>/bin/ai-cockpit --version`, and removal of the entire
  temporary root. The production documentation substitutes
  `https://github.com/xinglun/ai-cockpit.git` for the fixture URL. The local
  test does not create or depend on a remote tag.
- The second public release must add a real `brew upgrade` receipt. WI-34 tests
  upgrade state transitions with two synthetic Formula versions but does not
  fabricate a public upgrade receipt for the first release.

No job treats artifact upload, a semantic tag, or a provider Release alone as
proof that installation is complete.

## User Documentation Contract

English, Simplified Chinese, and Japanese documentation must cover:

- primary installation:
  `brew install xinglun/tap/ai-cockpit`;
- verification with `ai-cockpit --version`, SHA-256, and GitHub attestation;
- `brew update && brew upgrade xinglun/tap/ai-cockpit`;
- `brew uninstall ai-cockpit` and optional tap removal;
- manual macOS/Linux/Windows archive installation with checksum verification;
- tag-pinned, locked Cargo Git installation for Rust developers;
- rollback to a named immutable prior Release by verified manual archive,
  explicitly noting that the unversioned Formula tracks only the current
  release;
- MCP startup/configuration examples using the installed binary;
- the distinction between installing the `ai-cockpit` runtime and attaching a
  target repository. Installation does not create `.ai`; `attach` is a separate
  explicit operation.

## Error Handling and Publication States

The release state machine is:

`candidate -> locally_verified -> hosted_verified -> release_published -> tap_pr_open -> tap_merged -> install_verified`.

States never collapse. A failure after tag reservation does not permit reusing
or moving that tag. A Release without a merged Formula is publicly downloadable
but not Homebrew-complete. A merged Formula without successful installation
receipts is not GA-complete. Failed cross-repository handoff retains its exact
identity and requires a new authorized retry; it never falls back to direct
default-branch mutation.

WI-34 closes at a distinct `configuration_complete` readiness state after its
local fixtures, workflow policy tests, cross-target checks, documentation, and
independent review pass. It defines hosted jobs but does not claim their
results. WI-35 begins from that frozen candidate, obtains explicit hosted and
external-write authorization, and is solely responsible for
`hosted_verified` through `install_verified`. Thus an unexecuted hosted receipt
is reported as pending WI-35, not counted as missing WI-34 evidence.

## Scope Boundary

WI-34 may change release tooling, workflows, tests, and multilingual release
documentation in this repository. It may not create `.ai`, create the external
tap, commit, push, dispatch hosted workflows, create or move tags, publish a
Release, open a tap PR, merge, publish crates, or claim real public installation.

## Authoritative Sources

- [Homebrew: How to Create and Maintain a Tap](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)
- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Homebrew Tap Trust](https://docs.brew.sh/Tap-Trust)
- [Cargo install](https://doc.rust-lang.org/cargo/commands/cargo-install.html)
- [GitHub artifact attestations](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations)
- [GitHub release integrity verification](https://docs.github.com/en/code-security/how-tos/secure-your-supply-chain/secure-your-dependencies/verify-release-integrity)
- [GitHub-hosted runner labels](https://docs.github.com/en/actions/how-tos/write-workflows/choose-where-workflows-run/choose-the-runner-for-a-job)
