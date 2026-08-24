---
author: AI Cockpit maintainers
title: "WI-236 — v0.2.30 release baseline と public adopter acceptance"
workItemId: WI-236-release-v0-2-30
description: "マージ済み default branch から v0.2.30 を公開し、installed Runtime で immutable public artifact を検証します。"
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-236-release-v0-2-30
---

# WI-236 — v0.2.30 release baseline と public adopter acceptance

この Work Item は、マージ済み default branch から次の immutable public Runtime
Release を確立します。package identity と読者向け release 文書を更新した後、公開
archive、installed binary、adopter lifecycle、N-1 upgrade、isolation manifest、
finalization receipt を同じ Release identity に bind します。

## Acceptance boundary

- Workspace metadata と `Cargo.lock` が v0.2.30 を一貫して示す。
- Release、versioning、distribution、英中日 parity 文書が v0.2.30 を示し、v0.2.29
  を直前の N-1 baseline として保持する。
- 公開前の source quality、release policy、version consistency、documentation gate が pass する。
- 公開後に public Release tag と immutable artifact を検証し、source checkout や workspace
  binary を release evidence として使用しない。
- installed v0.2.30 binary の inspect/status/doctor/agent doctor と、isolated adopter/upgrade
  harness が pass する。temporary run root は cleanup し、acceptance receipt は監査可能に保持する。

## References

- [Release and Distribution](../release/distribution.ja.md)
- [Versioning](../architecture/versioning.ja.md)
- [Reference parity ledger](../reference/reference-parity.ja.md)
- [Public adopter acceptance harness](../../tests/release/adopter_acceptance.sh)
