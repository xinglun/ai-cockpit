---
author: AI Cockpit maintainers
title: "Release and Distribution"
description: "Reader-first installation, verification, upgrade, rollback, and MCP guidance."
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - release_distribution
keywords: [ai-cockpit, installation, release, homebrew, mcp]
---

# Release and Distribution

The public, identity-bound `v0.2.59` Release is the current installation baseline
after publication; before the provider Release exists, use the preceding public
`v0.2.58` archive. The reserved `v0.2.51` tag is an immutable failed publication
attempt (workflow run `33417057474`): it is a lightweight tag with no provider
Release and must never be reused or treated as an installation baseline.
The `v0.2.56` tag is also immutable failed publication history: its source-quality
workflow failed before a provider Release was created, so it is never reused or
treated as a public installation baseline.
Homebrew and manual installation use the published archive and manifest; the
repository configuration remains `cockpit.toml`, and installing the runtime
never creates `.ai` in a target repository. The same acceptance harness has a
staged-candidate mode before publication and a public-Release mode after
publication; neither mode obtains a Runtime from the source workspace.
The previous public `v0.2.58` baseline remains historical evidence after this
release and is not reused as the current installation identity. The earlier
public `v0.2.55`, `v0.2.53`, and `v0.2.52` baselines also remain historical evidence. The unpublished `v0.2.49` tag
(workflow run `33379366308`) is retained as immutable failed pre-publication
history and is not an installation baseline.

The `v0.2.46` tag is retained as immutable failed publication history
(`33330269507`); its public Release was never created and it is not an
installation baseline. The failure was caused by tagging before the mandatory
closed-Work-Item documentation promotion.

The unpublished `v0.2.36` tag remains immutable staged-acceptance failure
history and is not an installation baseline.

The `v0.2.35` tag is retained as failed publication history (workflow run
`33162800569`); it has no public Release or installable artifact and is not an
installation baseline. The earlier `v0.2.34` failure (workflow run
`33155382717`) remains preserved for the same reason.

The provider snapshot persisted with WI-239 and the current provider API both
report `immutable: false` for this Release. Release identity is therefore
drift-detectable, not provider-immutable: the tag, release manifest,
`SHA256SUMS`, archive digests, and post-release receipt must remain consistent.

The immutable `v0.2.30` tag records a release-route failure caused by an absent
active-Work-Item directory; it has no public Release and is preserved as
failed history. The reserved `v0.2.24` tag records a failed pre-publication governance gate,
the immutable `v0.2.25` tag records a source-quality failure, and `v0.2.26`
records a later release source-quality failure; none has a public Release.
They are immutable history, not installation baselines.

The `v0.2.32` tag is also retained as failed staged-publication history after
the adopter finalization binding defect fixed by WI-299; it has no public
Release and is not an installation baseline.

## CI quality and Runtime shadow boundary

CI uses versioned `repository_gate_manifest.json` as the canonical gate set. A
typed receipt selects cumulative `light`, `standard`, or `strict` coverage from
changed paths, Contract risk, and workflow stage. Unknown, release-owned,
high-risk, merge, and release inputs fail closed to `strict`. The runner
validates Git revisions, Contract and manifest digests, then executes only the
receipt's ordered gate IDs; it cannot accept an arbitrary command instead.

Release source quality always requests `strict`. Manifest-owned Cargo gates use
deterministic package-by-package tests, while CI and release upload both route
and gate receipts. `.gitattributes` excludes `.ai` and generated roots from the
source archive while retaining Cargo sources and lockfile.

The historical Runtime shadow baseline is pinned public `v0.2.28`; the
current release route additionally verifies `v0.2.59`. The
`tests/ci/runtime_verify_shadow.sh` receipt is an **execution smoke** for
standard/strict routes. It verifies identity-bound public `v0.2.59` and runs the
canonical repository profile. It does not claim Runtime-global T0–T3 routing,
affected-graph completeness, cross-Work-Item physical execution, or per-Work-
Item evidence coverage. The reference Makefile orchestration is different by
design and is not copied into this Rust repository. Runtime-global routing and
generic CLI `verify --command` semantics remain deferred outside WI-224's
non-`crates/**` scope.

## Before you start

You need a published, identity-bound Release, a repository path, and a matching
archive for your operating system. Homebrew installation requires Homebrew;
manual verification uses `shasum` and `awk` on macOS/Linux, and PowerShell on
Windows. `gh attestation verify` is an optional additional provenance check.

## Publishing a candidate

Publication is triggered by pushing an annotated Git tag after the reviewed
Work Item is merged and the default branch is synchronized. Create and push
the tag as follows; do not use `gh release create`, because it can create a
provider Release and a lightweight tag before the workflow has verified the
candidate:

```bash
git fetch origin main --tags
git tag -a v0.2.59 -m 'ai-cockpit v0.2.59'
test "$(git cat-file -t v0.2.59)" = tag
test "$(git rev-parse v0.2.59^{})" = "$(git rev-parse HEAD)"
git push origin v0.2.59
```

The workflow rejects a lightweight tag, an already existing provider Release,
or a tag whose peeled commit is not the reviewed source commit. A failed
publication reserves its tag permanently; the next candidate advances one
patch version.

## Primary macOS installation

When the maintained Homebrew tap is available, install the Formula from the
published release line:

```bash
brew install xinglun/tap/ai-cockpit
ai-cockpit --version
brew test xinglun/tap/ai-cockpit
```

Upgrade and uninstall are:

```bash
brew update
brew upgrade xinglun/tap/ai-cockpit
brew uninstall ai-cockpit
brew untap xinglun/tap                 # optional
```

The Formula currently targets macOS ARM64 and Intel. Linuxbrew is not a
supported path.

## Verify a Release asset

Download the archive, `release-manifest.json`, and `SHA256SUMS` from the same
published GitHub Release. The v0.2.59 checksum file covers all ten archive/SBOM
files, so validate the exact archive you downloaded:

```bash
archive="ai-cockpit-v0.2.59-aarch64-apple-darwin.tar.gz"
expected="$(awk -v name="$archive" '$2 == name {print $1}' SHA256SUMS)"
actual="$(shasum -a 256 "$archive" | awk '{print $1}')"
test -n "$expected" && test "$expected" = "$actual"
gh attestation verify "$archive" \
  --repo xinglun/ai-cockpit
```

If you use GitHub CLI after the Release exists, the equivalent download is:

```bash
archive="ai-cockpit-v0.2.59-aarch64-apple-darwin.tar.gz"
gh release download v0.2.59 --repo xinglun/ai-cockpit \
  --pattern "$archive" --pattern release-manifest.json --pattern SHA256SUMS
```

The filename, target, checksum, manifest, and attestation subject must agree.
Do not treat an upload or a semantic tag alone as installation evidence.
CLI and MCP `verify` JSON expose `runtimeVersion` and `runtimeDigest` as Runtime
identity facts. The post-release acceptance harness—not the Core by itself—must
bind those fields to the downloaded public binary before accepting release
evidence; a caller using the JSON outside that harness owns the comparison.

### Artifact-bound SBOM policy for later candidates

The failed staged v0.2.32 tag has no public assets to adopt. Its failure record
remains immutable and is not relabeled as a successful Release. For v0.2.59,
the public bytes become immutable once published: `SHA256SUMS` covers the five
archives and five target-named SBOMs, and each target SBOM is bound to its
packaged archive and executable as described below.

Release candidates built with the WI-241 boundary have a stricter contract.
Each target-named SPDX 2.3 document retains the dependency scan and adds one
release-archive Package plus one release-binary File. `DOCUMENT DESCRIBES` the
Package, the Package `CONTAINS` the File, and both nodes carry the nonzero
SHA-256 calculated from the actual staged archive and executable member. A
wrong target, version, filename, digest, node cardinality, or relationship
fails before candidate aggregation. The source dependency scan or an SBOM
filename alone is never adopter acceptance.

The closed public inventory is five archives, five target SBOMs,
`release-manifest.json`, `ai-cockpit.rb`, and `SHA256SUMS`. The manifest binds
the ten target artifacts; `SHA256SUMS` binds those ten plus the manifest and
Formula exactly once in stable filename order (it cannot checksum itself).
The final provenance subject set covers the same thirteen published files.
An extra build-named SBOM, other orphan publishable file, duplicate checksum
entry, missing entry, or digest mismatch fails closed. Existing staged/public
adopter acceptance and attestation gates remain downstream of this validation.

## Post-release adopter acceptance

Before publication, `staged_adopter_acceptance` binds the downloaded candidate
archive, manifest, and checksums to source `HEAD`, runs the canonical adopter
lifecycle and isolation checks, and proves cleanup. A separate
`staged_adopter_upgrade_acceptance` runs the previous public Release against
that staged target. Publication depends on both. Their receipts say
`stagedCandidate: true` and `releasePublished: false`; they do not rewrite
provider Release truth.

Maintainers can repeat the public-binary acceptance baseline after a Release.
The immutable `v0.2.36` tag currently records a failed staged acceptance and
has no public Release or adopter baseline. The repository-retained WI-239
receipt remains the historical v0.2.31 baseline. A successful future Release
must persist its own public-binary receipt before it is described as an
adopter baseline; hosted job artifacts alone are not a repository-persisted
baseline.

Persisted adopter acceptance baseline: `aarch64-apple-darwin` (WI-419,
public `v0.2.44`; binary digest
`sha256:69d28c970c2b89534e63cb685c6cc02a2f135d3067b6a84feaabce2adce1d5e5`).
The complete receipt is retained at
`.ai/evidence/WI-419-release-v0-2-44-adopter-acceptance/`. WI-416 remains the
immutable historical v0.2.43 baseline, and the earlier WI-239 receipt remains
historical v0.2.31 evidence; hosted job artifacts are not substituted for the
repository-persisted baseline.
GitHub Actions run `32696048024` remains separately retained as hosted Linux
acceptance evidence on `x86_64-unknown-linux-gnu`, not as this single-target
persisted baseline.

```bash
tests/release/adopter_acceptance.sh \
  --repository xinglun/ai-cockpit \
  --tag v0.2.59 \
  --target aarch64-apple-darwin \
  --output ./release-adopter-acceptance
```

The harness downloads only the named public Release, pins the extracted binary
by SHA-256, creates an isolated Cargo adopter, runs attach/profile/Agent doctor,
preserves `first-adopter-smoke` as `not_ready`, proves Work Item lifecycle and
evidence reuse, and emits `acceptance.json` plus `SHA256SUMS`. It never uses a
workspace or local Runtime binary. A failed post-release acceptance records
`releasePublished: true` and `adopterAcceptance: failed`; it does not rewrite
the already-published Release. Coverage for a second technology stack is a
separate future Work Item.

The lifecycle close in this post-release receipt is a complete structured Human
Decision. The harness requires the actor, authority source, reason, evidence
reference, policy reference, decision time, and resume condition. It copies the
regular, non-symlinked `.ai/decisions/<work-item>.close.json` into the
acceptance artifact and emits a binding record containing the adopter
`repositoryId`, Work Item ID, decision digest, and validation result. A missing,
foreign, incomplete, or mismatched close receipt fails closed; it cannot turn a
published Release back into an unpublished one.

Before either the old or new Work Item is closed, the harness exercises the
Runtime's resource-finalization boundary: `finalize-plan` binds the fixture's
branch/worktree context before verification. After archive, the harness
commits the archive on the fixture branch, fast-forwards the surviving control
worktree, removes the exact fixture branch and worktree, and only then records
`finalize` plus `finalize-verify` with `disposition: deleted`. This is a real
lifecycle requirement, not a cosmetic step; a retained resource must make
`close` fail closed.

After the receipt outputs are finalized, every success, failure, or interruption
path removes only its validated temporary `run_root`. `cleanup.json` and the
`cleanupState`/`cleanupError` fields in `acceptance.json` record the cleanup
result. A cleanup failure is fail-closed: the process exits non-zero and the
receipt becomes `adopterAcceptance: failed`, while `releasePublished` remains
true. Target and platform remain explicit, including the Linux x86_64 baseline
when that target is selected.

The N-1 harness returns zero only when both upgrade acceptance and cleanup pass;
an unset exit status is never treated as success.

The isolation receipt includes typed before/after manifests for files,
directories, symlinks, metadata, and digests. HOME and XDG_CONFIG_HOME are
forbidden-write roots. TMPDIR and CARGO_HOME are explicitly classified as
allowed Runtime-write roots; their writes are recorded rather than mistaken
for global configuration writes. Both public and N-1 harnesses resolve the
host `RUSTUP_HOME` and active toolchain before entering isolation, pass
`RUSTUP_TOOLCHAIN` explicitly, and refuse an implicit toolchain download.

To prevent a release from leaving configuration or documentation behind, the
release workflow derives the current version from Cargo metadata and runs
`tests/release/version_consistency.sh`. The source check validates all three
language routes and current archive examples; the post-release check validates
the public Release manifest and asset names. Historical N-1 references remain
explicit and are not mistaken for the current baseline.

The CI and release workflows pin every action to a full commit SHA. Their
Node-based actions use the official stable Node24-compatible baseline, and
`tests/release/action_runtime_policy.sh` checks both workflows for stale,
unpinned, or missing action refs. A future action-runtime change must update
that policy and this release note together.

### Historical N-1 schema migration acceptance

The schema-changing baseline is the historical v0.1.1 to v0.2.0 migration.
v0.2.59 is a same-schema patch release: its N-1 run follows the same harness
but records `migrationState: not_required` after compatibility is proven. To
reproduce a current N-1 run, use the immediately previous public Release and
the current Runtime:

```bash
tests/release/adopter_upgrade_acceptance.sh \
  --repository xinglun/ai-cockpit \
  --from-tag v0.2.58 \
  --to-tag v0.2.59 \
  --target aarch64-apple-darwin \
  --output ./release-adopter-upgrade-acceptance
```

This proves old-adopter detection, review-gated migration, byte-preserved
history, continued operation, and isolated repository/runtime identity. It is
post-release evidence and must never be replaced by a source build or used to
rewrite Release truth. The migration acceptance artifact is maintained
separately from the adopter installation path.

The historical v0.1.1 to v0.2.0 migration evidence remains archived. The
v0.2.0 Runtime predates the adjacent-chain receipt fields, so that historical
pair is not re-run by the current harness.

Before publication, the staged N-1 job uses a public N-1 archive and staged
candidate archive without a source build or arbitrary verification command.
After publication, the release workflow runs the public harness in a separate
`adopter_upgrade_acceptance` job after publication and the publication handoff.
For a tag push it resolves the immediately preceding published semantic
Release through the provider API. A first public Release records
`adopterAcceptance: not_applicable` with a checksummed receipt. Maintainers can
also trigger the workflow manually by supplying `from_tag`, `to_tag`, and an
optional `target`; manual dispatch consumes only those already-published
artifacts and never publishes a Release. The job uploads
`acceptance.json`, per-step JSON/stderr, both Runtime identity records, and
`SHA256SUMS` even when acceptance fails.

## Manual archive installation

macOS and Linux users download the matching `.tar.gz` and `SHA256SUMS`, choose
the exact Rust target, verify the archive, and place `ai-cockpit` in
`$HOME/.local/bin`:

```bash
target="aarch64-apple-darwin" # choose the target matching your machine
archive="ai-cockpit-v0.2.59-${target}.tar.gz"
expected="$(awk -v name="$archive" '$2 == name {print $1}' SHA256SUMS)"
actual="$(shasum -a 256 "$archive" | awk '{print $1}')"
test -n "$expected" && test "$expected" = "$actual"
mkdir -p "$HOME/.local/bin"
tar -xzf "$archive"
install -m 0755 ai-cockpit "$HOME/.local/bin/ai-cockpit"
case ":$PATH:" in
  *:"$HOME/.local/bin":*) ;;
  *) echo "Add $HOME/.local/bin to PATH before using ai-cockpit" >&2; exit 1 ;;
esac
"$HOME/.local/bin/ai-cockpit" --version
```

Windows users download the `.zip` and `SHA256SUMS`, compare the exact checksum,
extract it to a user bin directory, and add that directory to the user `PATH`:

```powershell
$archive = "ai-cockpit-v0.2.59-x86_64-pc-windows-msvc.zip"
$expected = Get-Content .\SHA256SUMS |
  Where-Object { ($_ -split '\s+')[1] -eq $archive } |
  ForEach-Object { ($_ -split '\s+')[0].ToLowerInvariant() }
$actual = (Get-FileHash .\$archive -Algorithm SHA256).Hash.ToLowerInvariant()
if ([string]::IsNullOrWhiteSpace($expected) -or $actual -ne $expected) { throw "Archive checksum mismatch" }
$destination = Join-Path $env:USERPROFILE "bin"
New-Item -ItemType Directory -Force -Path $destination | Out-Null
Expand-Archive .\$archive $destination -Force
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$userPath = if ([string]::IsNullOrEmpty($userPath)) { "" } else { $userPath }
if (($userPath -split ';') -notcontains $destination) {
  [Environment]::SetEnvironmentVariable("Path", ($userPath.TrimEnd(';') + ";" + $destination), "User")
}
$env:Path = "$destination;$env:Path"
& "$destination\ai-cockpit.exe" --version
```

## Rust developer fallback

This fallback is available for the current identity-bound `v0.2.59` tag.

After that publication, the workspace package must be selected explicitly:

```bash
cargo install --git https://github.com/xinglun/ai-cockpit.git \
  --tag v0.2.59 --locked --root "$HOME/.local" \
  --bin ai-cockpit cockpit-cli
"$HOME/.local/bin/ai-cockpit" --version
cargo uninstall --root "$HOME/.local" cockpit-cli
```

## Rollback

For rollback, download a named prior Release archive and verify its manifest and
digest before replacing the installed binary manually. The unversioned Homebrew
Formula tracks the current release; it is not a rollback selector.

## MCP and repository attachment

Start the local MCP adapter from the installed runtime with an explicit repository:

```bash
ai-cockpit mcp --repo /path/to/attached-repository
```

Installation and repository attachment are separate operations. Attach only
after reviewing the target Work Item:

```bash
ai-cockpit attach --repo /path/to/repository
```

For a person-facing MCP result, call `work_item_outcome` with an explicit
`workItemId` and optional `language`. Its text content is the same human
handoff as the CLI; `work_item_get` remains a raw machine lookup.

An illustrative MCP client configuration is:

```json
{
  "mcpServers": {
    "ai-cockpit": {
      "command": "ai-cockpit",
      "args": ["mcp", "--repo", "/path/to/attached-repository"]
    }
  }
}
```

Client configuration keys vary; the important contract is the installed binary,
the `mcp` subcommand, and an explicit repository path. Installation itself
does not attach or mutate a repository.
