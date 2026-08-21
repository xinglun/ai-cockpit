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

The public immutable `v0.1.1` Release is the current installation baseline.
Homebrew and manual installation use the published archive and manifest; the
repository configuration remains `cockpit.toml`, and installing the runtime
never creates `.ai` in a target repository. WI-40 adds a post-release adopter
acceptance harness; it is not a pre-publication gate or a Runtime command.

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

The Formula currently targets macOS ARM64 and Intel. Linuxbrew is not a WI-34
supported path.

## Verify a Release asset

Download the archive, `release-manifest.json`, and `SHA256SUMS` from the same
immutable GitHub Release. The checksum file covers all ten archive/SBOM files,
so validate the exact archive you downloaded:

```bash
archive="ai-cockpit-v0.1.1-aarch64-apple-darwin.tar.gz"
expected="$(awk -v name="$archive" '$2 == name {print $1}' SHA256SUMS)"
actual="$(shasum -a 256 "$archive" | awk '{print $1}')"
test -n "$expected" && test "$expected" = "$actual"
gh attestation verify "$archive" \
  --repo xinglun/ai-cockpit
```

If you use GitHub CLI after the Release exists, the equivalent download is:

```bash
archive="ai-cockpit-v0.1.1-aarch64-apple-darwin.tar.gz"
gh release download v0.1.1 --repo xinglun/ai-cockpit \
  --pattern "$archive" --pattern release-manifest.json --pattern SHA256SUMS
```

The filename, target, checksum, manifest, and attestation subject must agree.
Do not treat an upload or a semantic tag alone as installation evidence.

## Post-release adopter acceptance

Maintainers can repeat the public-binary acceptance baseline after a Release:

```bash
tests/release/adopter_acceptance.sh \
  --repository xinglun/ai-cockpit \
  --tag v0.1.1 \
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

### N-1 upgrade acceptance

When a new Runtime changes the Repository Protocol schema, run the separate
N-1 harness with both public archives:

```bash
tests/release/adopter_upgrade_acceptance.sh \
  --repository xinglun/ai-cockpit \
  --from-tag v0.1.1 \
  --to-tag v0.2.0 \
  --target aarch64-apple-darwin \
  --output ./release-adopter-upgrade-acceptance
```

This proves old-adopter detection, review-gated migration, byte-preserved
history, continued operation, and isolated repository/runtime identity. It is
post-release evidence and must never be replaced by a source build or used to
rewrite Release truth. See [WI-44](../work-items/WI-44-n-minus-one-upgrade-acceptance.md).

## Manual archive installation

macOS and Linux users download the matching `.tar.gz` and `SHA256SUMS`, choose
the exact Rust target, verify the archive, and place `ai-cockpit` in
`$HOME/.local/bin`:

```bash
target="aarch64-apple-darwin" # choose the target matching your machine
archive="ai-cockpit-v0.1.1-${target}.tar.gz"
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
$archive = "ai-cockpit-v0.1.1-x86_64-pc-windows-msvc.zip"
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

This fallback is available for the current immutable `v0.1.1` tag.

After that publication, the workspace package must be selected explicitly:

```bash
cargo install --git https://github.com/xinglun/ai-cockpit.git \
  --tag v0.1.1 --locked --root "$HOME/.local" \
  --bin ai-cockpit cockpit-cli
"$HOME/.local/bin/ai-cockpit" --version
cargo uninstall --root "$HOME/.local" cockpit-cli
```

## Rollback

For rollback, download and verify a named immutable prior Release archive and
replace the installed binary manually. The unversioned Homebrew Formula tracks
the current release; it is not a rollback selector.

## MCP and repository attachment

Start the local MCP adapter from the installed runtime:

```bash
ai-cockpit mcp
ai-cockpit mcp --repo /path/to/attached-repository
```

Installation and repository attachment are separate operations. Attach only
after reviewing the target Work Item:

```bash
ai-cockpit attach --repo /path/to/repository
```

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
