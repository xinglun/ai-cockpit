---
author: AI Cockpit maintainers
title: "WI-236 — v0.2.30 release baseline と public adopter acceptance"
workItemId: WI-236-release-v0-2-30
description: "マージ済み default branch から v0.2.30 を公開し、installed Runtime で immutable public artifact を検証します。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-236-release-v0-2-30
---

# WI-236 — v0.2.30 release baseline と public adopter acceptance

この Work Item は、次の immutable public Runtime Release の公開前 baseline を確立します。
package identity と読者向け release 文書を更新し、review 済み PR と pre-merge
finalization boundary を bind します。public artifact identity、installed binary、
adopter lifecycle、N-1 upgrade は merge 後の事実であり、公開後に successor Work Item
が検証します。この Work Item はそれらを先取りして主張しません。

## Acceptance boundary

- Workspace metadata と `Cargo.lock` が v0.2.30 を一貫して示す。
- Release、versioning、distribution、英中日 parity 文書が v0.2.30 を示し、v0.2.29
  を直前の N-1 baseline として保持する。
- 公開前の source quality、release policy、version consistency、documentation gate が pass する。
- review 済み PR に有効な pre-merge finalization boundary があり、Release tag は merge 後に
  だけ作成する。source checkout や workspace binary を public release evidence として使用しない。
- successor Work Item が公開後に installed v0.2.30 binary と isolated adopter/upgrade harness
  を検証する。temporary run root は cleanup し、successor の acceptance receipt を監査可能に保持する。

## References

- [Release and Distribution](../release/distribution.ja.md)
- [Versioning](../architecture/versioning.ja.md)
- [Reference parity ledger](../reference/reference-parity.ja.md)
- [Public adopter acceptance harness](../../tests/release/adopter_acceptance.sh)
