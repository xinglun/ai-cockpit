---
author: AI Cockpit maintainers
title: "WI-241 — Release SBOM artifact binding"
workItemId: WI-241-release-sbom-binding
description: "以降の各 target SBOM を正確な packaged bytes に binding し、public Release asset inventory を閉じます。"
audience:
  - maintainer
  - reviewer
  - adopter
status: implemented
lastVerifiedBy: WI-241-release-sbom-binding
authority: canonical
---

# WI-241 — Release SBOM artifact binding

WI-241 は v0.2.31 enterprise-compliance audit で見つかった Release construction
boundary を修正します。対象は以降の candidate だけです。公開済み v0.2.31 tag、
Release assets、checksums、attestations、acceptance receipts は immutable な
historical truth のままです。

## Delivered boundary

- `cockpit-release bind-sbom` は実際の staged archive と、その archive から読み取った
  executable member の SHA-256 を計算します。標準 SPDX 2.3 release Package と File を
  挿入し、`DESCRIBES` と `CONTAINS` で結び、write 前に document 全体を検証します。
- validator は正確な target、canonical version、target-named archive/SBOM、一つの
  reserved Package、一つの reserved File、各一つの binding relationship、matching
  nonzero SHA-256 を要求します。
- Anchore dependency scan は保持しますが、automatic artifact/Release upload は無効です。
  candidate、attestation、publication allowlist に入れる SBOM は五つの target-named
  file だけです。
- checksum は Formula の後に生成します。五つの archive、五つの SBOM、canonical
  manifest、Formula を stable order で一度ずつ対象にします。checksum file 自体は
  十三番目の public asset であり、自身を checksum できません。
- candidate validation は downstream の staged adopter acceptance 前に、missing/orphan
  publishable asset、duplicate checksum name、unsorted/malformed line、missing/extra entry、
  digest mismatch を拒否します。

## Evidence boundary

SPDX filename や dependency scan は adopter acceptance ではありません。SBOM が証明するのは
正確な archive/binary binding だけです。Hosted attestation と既存の staged/public adopter
acceptance job は独立した downstream gate のままです。

Regression coverage は `crates/cockpit-release/tests/sbom.rs`、
`crates/cockpit-release/tests/manifest.rs`、`tests/release/workflow_policy.sh` にあります。
Runtime verification は `.ai/evidence/WI-241-release-sbom-binding.verification.json` に記録します。

## References

- [Release distribution](../release/distribution.ja.md)
- [Reference source parity](../reference/reference-parity.ja.md)
