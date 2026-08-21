---
author: AI Cockpit maintainers
title: "Release Distribution Architecture"
description: "検証済み Rust build を install 可能な AI Cockpit runtime にする方法と、installation と attach の境界。"
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - release_distribution
keywords: [ai-cockpit, release, homebrew, distribution, provenance]
---

# Release Distribution Architecture

## 目的

このページは、**release で何を trust し、どう runtime を install し、Homebrew がどこで止まるか**を説明します。

## 対象読者

AI Cockpit を install する前、または release pipeline を review する前に読んでください。adopter
向けの説明を先にし、maintainer が確認する identity binding も示します。

## 読後の理解

source of truth となる artifact、五つの target の binding、tap handoff の範囲、installation が
repository を黙って attach しない理由を理解できます。

## Release と installation の flow

```text
source commit + immutable tag
            │
            ▼
source quality + policy gate
            │
            ▼
五つの target build（archive + SBOM）
            │
            ▼
canonical manifest + SHA256SUMS
            │
            ▼
artifact smoke test + provenance attestation
            │
            ▼
        GitHub Release
       ┌────┼───────────────┬─────────────────┐
       ▼    ▼               ▼                 ▼
 Homebrew  verified       Cargo Git        manual archive
 Formula   archive        fallback          install
       │    │               │                 │
       └────┴───────────────┴─────────────────┘
                         ▼
                   `ai-cockpit`
                         │ explicit attach
                         ▼
       対象 repository + `.ai/cockpit.toml` + `.ai/project.json`

homebrew-handoff.json ──► WI-35 verifier ──► tap PR
                          (external、WI-34 の外部)
```

Release manifest は version、tag、commit、target、runner image、archive、SBOM、bytes、digest、
provenance subject を binding します。`SHA256SUMS` は manifest にある archive と SBOM だけを対象にします。
provider Release や単独の artifact upload だけでは installation evidence になりません。

## Adopter が行うこと

1. 公開済み Homebrew Formula から install するか、immutable Release から対応する archive を download します。
2. version、SHA-256 digest、provider attestation を verify します。
3. 対象 repository と Work Item を review してから、`ai-cockpit attach --repo /path/to/repository`
   を実行します。attach は明示的な手順で、`.ai/` を作成・更新し得ます。
4. attach 済み repository に対して CLI または MCP adapter を起動します。

candidate を build する development checkout は `.ai` を持ちません。`cockpit.toml` は `.ai/` 配下の TOML のままであり、
distribution が JSON へ移行させることもありません。

## Trust boundary

- WI-34 は local release contract、deterministic な manifest と Formula、staged install test、identity-bound
  handoff document を所有します。
- WI-35 は hosted release receipt、immutable tag と provider Release、external tap、tap pull request、
  real public install receipt を所有します。
- Tap は review 済み Formula projection を受け取り、binary を rebuild しません。
- Homebrew は delivery path であり governance authority ではありません。repository facts と human decision
  は attach 済み repository と Work Item から来ます。

## Stop conditions

tag、workspace version、binary version、commit、manifest、digest、SBOM、provenance subject、provider Release
identity が一致しない場合は停止します。handoff が expired、別 commit を指す、別 destination を要求する、
default branch を直接変更しようとする場合も停止します。installation を repository attach の証明として扱う
場合も同様です。

## 次に読むもの

1. [Release と配布](../release/distribution.ja.md) — adopter command。
2. [Architecture](../architecture.ja.md) — runtime と evidence ownership。
3. [WI-34](../work-items/WI-34.ja.md) — local readiness と external boundary。

## 技術的な深さ

Rust `cockpit-release` package は manifest、archive、Formula、handoff を strict に検証します。GitHub Actions
は五つの retained target を build し、source、verification、attestation、publication、handoff の権限を分離し、
external tap mutation を default repository token の外側に置きます。
