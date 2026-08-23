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

The public immutable `v0.2.22` Release is the current installation baseline.
Homebrew and manual installation use the published archive and manifest; the
repository configuration remains `cockpit.toml`, and installing the runtime
never creates `.ai` in a target repository. A post-release adopter acceptance
harness is available for maintainers; it is not a pre-publication gate or a
Runtime command.

## CI quality and Runtime shadow boundary

The release source-quality gate uses the same deterministic package-by-package
test strategy as CI. Each workspace package is tested with
`cargo test -p <package> --all-targets -- --test-threads=1`; Cargo test
binaries are not launched concurrently. The verifier's own declared worker
cap can still exercise parallel commands inside a test binary. This keeps the
release gate aligned with CI without removing the Cargo checks.

The `tests/ci/runtime_verify_shadow.sh` receipt is a Phase 1 **execution
smoke**. It downloads and verifies an immutable public Runtime, then proves
that it can execute one repository-bound verification command. Its receipt
explicitly does not claim policy-route or planner coverage, affected-graph
completeness, cross-Work-Item physical execution, or per-Work-Item evidence
receipt coverage. Those claims require the corresponding Runtime and external
evidence gates; a passing shadow is not a substitute for them.

## Before you start

You need a published immutable Release, a repository path, and a matching
archive for your operating system. Homebrew installation requires Homebrew;
manual verification uses `shasum` and `awk` on macOS/Linux, and PowerShell on
Windows. `gh attestation verify` is an optional additional provenance check.

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
immutable GitHub Release. The checksum file covers all ten archive/SBOM files,
so validate the exact archive you downloaded:

```bash
archive="ai-cockpit-v0.2.22-aarch64-apple-darwin.tar.gz"
expected="$(awk -v name="$archive" '$2 == name {print $1}' SHA256SUMS)"
actual="$(shasum -a 256 "$archive" | awk '{print $1}')"
test -n "$expected" && test "$expected" = "$actual"
gh attestation verify "$archive" \
  --repo xinglun/ai-cockpit
```

If you use GitHub CLI after the Release exists, the equivalent download is:

```bash
archive="ai-cockpit-v0.2.22-aarch64-apple-darwin.tar.gz"
gh release download v0.2.22 --repo xinglun/ai-cockpit \
  --pattern "$archive" --pattern release-manifest.json --pattern SHA256SUMS
```

The filename, target, checksum, manifest, and attestation subject must agree.
Do not treat an upload or a semantic tag alone as installation evidence.
CLI and MCP `verify` JSON expose `runtimeVersion` and `runtimeDigest` as Runtime
identity facts. The post-release acceptance harness—not the Core by itself—must
bind those fields to the downloaded public binary before accepting release
evidence; a caller using the JSON outside that harness owns the comparison.

## Post-release adopter acceptance

Maintainers can repeat the public-binary acceptance baseline after a Release:

**Complete adopter acceptance baseline: `x86_64-unknown-linux-gnu` for v0.2.22.**
The other four published targets have build and
smoke evidence in the Release workflow; they are not claimed to have completed
the full adopter lifecycle unless a separate acceptance run is recorded.

```bash
tests/release/adopter_acceptance.sh \
  --repository xinglun/ai-cockpit \
  --tag v0.2.22 \
  --target x86_64-unknown-linux-gnu \
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
branch/worktree context before verification, and the post-archive `finalize`
plus `finalize-verify` receipt records the intentionally retained fixture
resources. This is a real lifecycle requirement, not a cosmetic step; omitting
it must make `close` fail closed.

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
v0.2.22 is a same-schema patch release: its N-1 run follows the same harness
but records `migrationState: not_required` after compatibility is proven. To
reproduce a current N-1 run, use the immediately previous public Release and
the current Runtime:

```bash
tests/release/adopter_upgrade_acceptance.sh \
  --repository xinglun/ai-cockpit \
  --from-tag v0.2.20 \
  --to-tag v0.2.22 \
  --target x86_64-unknown-linux-gnu \
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

The release workflow runs this harness in a separate
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
archive="ai-cockpit-v0.2.22-${target}.tar.gz"
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
$archive = "ai-cockpit-v0.2.22-x86_64-pc-windows-msvc.zip"
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

This fallback is available for the current immutable `v0.2.22` tag.

After that publication, the workspace package must be selected explicitly:

```bash
cargo install --git https://github.com/xinglun/ai-cockpit.git \
  --tag v0.2.22 --locked --root "$HOME/.local" \
  --bin ai-cockpit cockpit-cli
"$HOME/.local/bin/ai-cockpit" --version
cargo uninstall --root "$HOME/.local" cockpit-cli
```

## Rollback

For rollback, download and verify a named immutable prior Release archive and
replace the installed binary manually. The unversioned Homebrew Formula tracks
the current release; it is not a rollback selector.

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
