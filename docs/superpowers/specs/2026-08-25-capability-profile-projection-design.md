# Rust-native Capability and Profile Projection

## Status

Approved by the repository owner as the next reference-comparison batch.

## Goal

Close the four capability/profile comparison records without copying the
reference Python runtime, Make targets, provider-global configuration, or a
static installed-runtime manifest into the repository.

## Design

The shared Rust Runtime remains external and request-scoped. Repository-owned
declarations are optional, explicit, human-authored inputs. The Runtime reads
them through a no-write projection and binds them to the repository identity
and the current snapshot.

### Repository declarations

- `.ai/project/capabilities.json` declares capabilities, non-capabilities,
  critical domains, and operation-to-capability mappings.
- `.ai/project/success_criteria.json` is a compatibility projection only. A
  Work Item Contract's acceptance criteria remain authoritative; project-level
  criteria cannot approve, complete, or replace a Work Item.
- `.ai/project/profile-policy.json` is the Rust-native JSON projection of the
  reference Project Profile policy surface. It holds optional approved
  boundaries, critical paths, review requirements, and explicit unknowns. The
  existing `.ai/project.json` remains the strict identity and observed-quality
  profile; no YAML parser or `project_profile.yaml` copy is introduced.

All three declarations are strict schemas, regular-file-only, repository
identity-bound, and carry the reviewed `repositorySnapshotDigest`; Runtime
output also exposes canonical semantic declaration digests. `attach` does not
invent capabilities, boundaries, authority, or success decisions. `profile propose`
may emit a candidate projection; only explicit human confirmation may update a
formal declaration.

### Capability and adopter surface output

`capability show` and MCP `capability_show` continue to return the dynamic
Runtime registry. The projection adds declaration/profile digests, visible
non-authoritative success criteria, and a surface state that distinguishes
Runtime support, repository binding, and
external Release/adopter evidence. It must never claim `adopter_installed`
from file presence alone.

### Preflight binding

When a Contract declares `operation` or `requestedOperation`, the Runtime
checks the repository capability mapping. Missing, malformed, foreign, stale,
conflicting, or insufficient mappings add stable unknowns and produce a human
review state. The Runtime never derives a capability from intent prose or
model output. Contracts without an operation retain legacy compatibility.

Success criteria are checked only for identity and visibility; they never
override Contract acceptance or create approval.

## Safety and isolation

- All reads are explicit `--repo` and no process-global project state exists.
- Symlinks, parent paths, unknown fields, duplicate keys, foreign repository
  IDs, and digest mismatches fail closed.
- Two repositories may project concurrently with independent declarations and
  snapshots.
- Projection commands do not write generated status, evidence, or declarations.

## Verification

Focused Rust tests cover valid declarations, malformed/foreign/stale inputs,
operation coverage, legacy contracts, success-criteria precedence, surface
identity, symlink rejection, read-only behavior, and parallel repository
isolation. The full workspace, documentation, conformance, and hosted gates
remain required before merge.
