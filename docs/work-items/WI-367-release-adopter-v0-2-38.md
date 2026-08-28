---
author: AI Cockpit maintainers
title: "WI-367 — v0.2.38 public release adopter acceptance"
workItemId: WI-367-release-adopter-v0-2-38
description: "Accept the immutable public v0.2.38 artifact in an isolated fresh adopter repository and preserve a repeatable evidence baseline."
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-367-release-adopter-v0-2-38
terminalArchive: .ai/work-items/archive/WI-367-release-adopter-v0-2-38.contract.json
terminalVerification: .ai/evidence/WI-367-release-adopter-v0-2-38.verification.json
terminalFinalization: .ai/decisions/WI-367-release-adopter-v0-2-38.finalize.json
terminalDecision: .ai/decisions/WI-367-release-adopter-v0-2-38.close.json
capabilityClaims: [release_distribution, adopter_acceptance, repository_isolation]
---

# WI-367 — v0.2.38 public release adopter acceptance

[简体中文](WI-367-release-adopter-v0-2-38.zh-CN.md) · [日本語](WI-367-release-adopter-v0-2-38.ja.md)

## Intent

Use only the immutable public v0.2.38 Release binary to govern a fresh adopter
repository, prove the release acceptance and isolation boundaries, and retain a
repeatable baseline for future releases.

## Scope and boundary

- Run the public adopter acceptance and upgrade harnesses without source,
  workspace-binary, or `cargo build`/`cargo run` fallback.
- Preserve runtime identity, release metadata, evidence reuse, complete Work
  Item lifecycle receipts, isolation manifests, and cleanup proof.
- Keep the manifest helper portable across macOS Bash 3.2 and Linux Bash.

Runtime implementation changes, CI workflow policy, global Agent/MCP
configuration, historical evidence rewrites, and adopter business code are
outside this Work Item.

## Acceptance

1. Public v0.2.38 release metadata, archive digest, and binary digest are
   captured and mutually consistent.
2. A fresh adopter repository is attached and governed only by the downloaded
   v0.2.38 binary.
3. Repository identity, Work Item lifecycle, evidence reuse, and close
   decision are recorded with runtime identity.
4. HOME and XDG_CONFIG_HOME remain unchanged; runtime-write roots are isolated
   and the temporary run root is removed on success and failure.
5. No source checkout, workspace binary, or Cargo fallback is used.
6. Acceptance receipts and checksums are reproducible and suitable for future
   release baselines.
7. The public harness completes on macOS Bash 3.2 and Linux Bash without
   manifest deadlock.

## Evidence and result

- Public Release: [v0.2.38](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.38)
- Release workflow: [33195494850](https://github.com/xinglun/ai-cockpit/actions/runs/33195494850)
- Acceptance evidence: `.ai/evidence/WI-367-release-adopter-v0-2-38/acceptance.json`
- Runtime verification: `.ai/evidence/WI-367-release-adopter-v0-2-38.verification.json`
- Isolation and cleanup evidence: `.ai/evidence/WI-367-release-adopter-v0-2-38/isolation.json` and `cleanup.json`

The public workflow and the local immutable-artifact run passed. The
macOS-portability defect found during the first run was corrected in
`tests/release/isolation_manifest.sh` and covered by its regression test.
