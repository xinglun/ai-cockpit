---
author: AI Cockpit maintainers
title: "WI-151 — v0.2.16 post-release self-governance acceptance"
description: "Use only the immutable public v0.2.16 binary to verify AI Cockpit governing this repository after installation."
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-151-post-release-v0-2-16-self-governance
workItemId: WI-151-post-release-v0-2-16-self-governance
---

# WI-151 — v0.2.16 post-release self-governance acceptance

WI-151 is the post-release acceptance boundary. It downloaded the public
v0.2.16 aarch64 macOS archive, verified its checksum and archive layout, and
installed the extracted binary without a source or workspace fallback.

The installed binary identity was:

- version: `0.2.16`
- binary SHA-256: `0e9e9e85f3a96d22702cf95edab928bd2307c4636e53836bee46ca4e8cabf796`
- repositoryId: `sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b`

With an explicit `--repo`, `inspect`, `status`, `doctor`, `agent doctor`, and a
full workspace verification passed. The human Outcome was rendered in English,
Simplified Chinese, and Japanese with the visible `🟢` marker and a structured
Human Decision. The acceptance evidence is
`.ai/evidence/WI-151-post-release-v0-2-16-self-governance.verification.json`;
the decision is
`.ai/decisions/WI-151-post-release-v0-2-16-self-governance.close.json`.

The release workflow and public artifact remain the authoritative publication
evidence; this Work Item records the installed-runtime adopter result.
