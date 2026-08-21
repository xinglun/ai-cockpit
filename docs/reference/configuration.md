---
author: AI Cockpit maintainers
title: "Configuration Reference"
description: "Repository-owned TOML configuration, profile state, and generated Work Item files."
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - configuration
---

# Configuration reference

The repository configuration format is TOML. It is not changed to JSON.

## `.ai/cockpit.toml`

`attach` creates a minimal file:

```toml
protocol_version = 1
repository_id = "sha256:<64 lowercase hexadecimal characters>"
```

The runtime validates both fields and rejects an identity mismatch. Do not copy
runtime source or V1 files into `.ai/`.

## `.ai/project.json`

`attach` creates an attached profile with `state: "calibration_required"`.
After `profile confirm`, the profile version increments and the selected quality
command is recorded as verified. The wrapper contains `profileVersion`,
`repositoryId`, `state`, `profileDigest`, `tests`, and `buildSystems`. Unknown
profile fields are rejected.

## Work Item records

`start` generates these files under `.ai/work-items/active/`:

- `<id>.contract.json` — intent, scope, authority, acceptance, required evidence,
  base revision, profile digest, and repository snapshot digest;
- `<id>.summary.json` — lifecycle state and checkpoint count.

`verify --work-item <id>` writes `.ai/evidence/<id>.verification.json`. `finish`
creates an outcome, `archive` creates an archive manifest, and `close` records the
human decision. These records are content-bound and must not be hand-edited to make
a decision appear green.

Cross-process reusable evidence is runtime-managed under
`.ai/evidence/reuse/`; see [Protocol v1](../protocol/v1/specification.md) for its
schema, identity bindings, and resource limits.
