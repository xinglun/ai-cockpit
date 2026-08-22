---
author: AI Cockpit maintainers
title: "WI-111 MCP Human Handoff and Release Isolation Evidence"
description: "Repository-bound human Outcome delivery and typed post-release isolation manifests."
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: mcp-isolation-regression
capabilityClaims:
  - mcp_human_outcome_handoff
  - typed_release_isolation_evidence
---

# WI-111: MCP human handoff and release isolation evidence

## Goal

Make the human-facing Outcome a first-class Agent handoff and make release
adopter isolation evidence strong enough to detect file, directory, symlink,
metadata, and digest changes without weakening cleanup or repository binding.

## Scope

The repository service owns one human Outcome renderer. CLI and MCP call that
same renderer after `outcome_v2` validation. MCP adds the explicit,
repository-bound `work_item_outcome` tool; `work_item_get` remains a raw machine
record lookup. The tool returns stable `structuredContent.outcome` plus a
visible localized `humanHandoff`. Contract source text is preserved and no
human decision is inferred.

Release adopter and upgrade harnesses share a typed isolation manifest. Each
manifest records relative path, entry type, mode/size/mtime metadata, and a
SHA-256 digest for regular files or symlink targets. HOME and XDG_CONFIG_HOME
are forbidden-write roots; TMPDIR and CARGO_HOME are explicitly classified
runtime-write roots. The receipt binds before/after manifest digests and the
validated temporary-root cleanup result.

## Acceptance

- CLI and MCP use the same renderer and expose status marker, unknowns,
  evidence, structured human decision projection, and next action.
- English, Chinese, and Japanese MCP handoffs are covered; Contract criteria
  remain in their original language.
- Manifest regression covers file content, directory, symlink target, and
  metadata mutations, plus cleanup with no residual run root.
- Public v0.2.7 adopter acceptance passes with `isolation.json` schema 2,
  typed manifests, `cleanup.json`, and directory `SHA256SUMS`.
- Repository-local Agent instructions describe the handoff and isolation
  boundaries without changing global Agent or MCP configuration.

## Verification

```text
cargo test --locked -p cockpit-mcp --test rpc -- --test-threads=1
cargo test --locked -p cockpit-cli --test intelligence --test outcome_human_decision -- --test-threads=1
bash tests/release/isolation_manifest_test.sh
bash tests/release/adopter_acceptance_test.sh
bash tests/release/adopter_upgrade_acceptance_test.sh
bash tests/docs/documentation_acceptance.sh
```

The public acceptance run uses only the downloaded v0.2.7 binary; it does not
fall back to source or a workspace binary. Future work remains separately
responsible for strict typed verification evidence, foreign-runtime policy,
historical evidence projection, and external immutable audit retention.

## Outcome

Status: **Implemented locally; focused MCP, CLI, documentation, manifest, and
public adopter acceptance checks passed.**
