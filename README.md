---
author: AI Cockpit maintainers
title: "AI Cockpit"
description: "Repository governance for AI coding agents: explicit scope, verification evidence, and auditable human decisions. Built in Rust with CLI and Model Context Protocol (MCP) interfaces."
audience:
  - adopter
  - contributor
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - repository_governance_layer
---

# AI Cockpit

[中文](README.zh-CN.md) | [日本語](README.ja.md)

Repository governance for AI coding agents. AI Cockpit makes scope explicit, keeps verification evidence reviewable, and records human decisions separately. Built in Rust with a command-line interface and a local Model Context Protocol (MCP) adapter.

## When an agent says “done”

Did the change stay within scope? Were required checks run against the current repository? Is approval still valid for the change being reviewed?

Before work begins, a human records a [Work Item Contract](docs/getting-started/first-work-item.md). It states the task, what the agent may change, how success is checked, and who can approve the result. Verification records evidence for those checks and the repository state. The human decision stays explicit and auditable; passing checks does not approve the work.

Each repository stores its own governance state in its `.ai/` directory.

For an ordinary repository-only change, `start --prepare` records the Contract and runs preflight, a readiness check for the repository and requested work. When no human decision is required, it saves a checkpoint: a repository snapshot taken before editing.

After implementation, `verify` records evidence for the Contract's declared checks. `finish` checks that the evidence matches the current repository snapshot, then records the Work Item Outcome. `archive` preserves the Work Item record.

Keep verification and audit evidence. Follow the active Work Item's Runtime next action. If work uses only a local branch or worktree, `close` records the decision before that local cleanup. If a provider owns the PR, branch, or worktree, complete and verify its declared cleanup before `close`. Never delete a worktree that holds the only copy of the evidence. See the [Agent workflow reference](docs/reference/agent-workflow.md) for details.

## Try a real first-use route

For Apple Silicon macOS, install the stable v1.0.1 binary after publication. The command verifies the downloaded archive against the release manifest and SHA256SUMS. It requires curl, shasum, tar, install, and Python 3:

~~~bash
set -eu
tag=v1.0.1
release_url="https://github.com/xinglun/ai-cockpit/releases/download/$tag"
asset="ai-cockpit-$tag-aarch64-apple-darwin.tar.gz"
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"
curl -fL "$release_url/$asset" -o "$asset"
curl -fL "$release_url/SHA256SUMS" -o SHA256SUMS
curl -fL "$release_url/release-manifest.json" -o release-manifest.json
manifest_sum="$(awk '$2 == "release-manifest.json" {print $1}' SHA256SUMS)"
manifest_actual="$(shasum -a 256 release-manifest.json | awk '{print $1}')"
test -n "$manifest_sum" && test "$manifest_actual" = "$manifest_sum"
expected="$(python3 - "$asset" <<'PY'
import json
import sys
filename = sys.argv[1]
with open("release-manifest.json", encoding="utf-8") as stream:
    manifest = json.load(stream)
if manifest.get("version") != "1.0.1" or manifest.get("tag") != "v1.0.1":
    raise SystemExit("release manifest identity mismatch")
record = next((item for item in manifest["artifacts"]
               if item["archive"]["filename"] == filename
               and item["target"] == "aarch64-apple-darwin"), None)
if record is None:
    raise SystemExit("Apple Silicon macOS artifact missing from manifest")
print(record["archive"]["sha256"])
PY
)"
listed="$(awk -v name="$asset" '$2 == name {print $1}' SHA256SUMS)"
actual="$(shasum -a 256 "$asset" | awk '{print $1}')"
test -n "$expected" && test "$expected" = "$listed" && test "$actual" = "$listed"
tar -xzf "$asset" ai-cockpit
mkdir -p "$HOME/.local/bin"
install -m 0755 ai-cockpit "$HOME/.local/bin/ai-cockpit"
export PATH="$HOME/.local/bin:$PATH"
ai-cockpit --version
~~~

Add $HOME/.local/bin to your shell startup PATH to use the command in new terminals.

This command targets Apple Silicon macOS only. For Linux ARM64 GNU, Linux x86_64 GNU, or Windows x86_64, use the [distribution guide](docs/release/distribution.md) for exact release assets and checksums. Then choose a repository path you control. In a POSIX shell, run:

~~~bash
repo=/path/to/repository
ai-cockpit --version
ai-cockpit inspect --repo "$repo"
ai-cockpit attach --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
~~~

In Windows PowerShell, set the repository path and run the same first-use steps:

~~~powershell
$repo = "C:\path\to\repository"
ai-cockpit --version
ai-cockpit inspect --repo $repo
ai-cockpit attach --repo $repo
ai-cockpit status --repo $repo
ai-cockpit doctor --repo $repo
~~~

**inspect** reads repository facts. **attach** initializes the repository-local `.ai/` directory for governance state; it does not install Agent instructions or change global MCP settings. **status** summarizes repository state and Runtime compatibility. **doctor** checks repository attachment, protocol version, and Runtime compatibility. Review the output: install or attach alone does not mean work is approved or verified. See the [first Work Item walkthrough](docs/getting-started/first-work-item.md).

## Stable release and optional prerelease

Use stable v1.0.1 by default. Its [release page](https://github.com/xinglun/ai-cockpit/releases/tag/v1.0.1) lists the Apple Silicon macOS, Linux ARM64 GNU, Linux x86_64 GNU, and Windows x86_64 artifacts. The preceding stable v1.0.0 Release remains the N-1 upgrade source. The v1.0.1-rc.2 prerelease remains a separate historical release and is not the default installation. No Intel macOS, Linux musl, or Windows ARM64 archive is provided.

## Boundaries

AI Cockpit records scope, declared verification, evidence freshness, and human decisions. It does not provide a production sandbox, configure branch protection, prove external provider identity, or replace human review and security policy. A recorded check is evidence about that check and repository state, not a claim of universal safety or improved performance.

## Continue

- [Getting started](docs/getting-started/README.md)
- [Capabilities and boundaries](docs/capabilities.md)
- [Release and distribution](docs/release/distribution.md)
- [Agent workflow reference](docs/reference/agent-workflow.md)
- [Contributing](CONTRIBUTING.md)
