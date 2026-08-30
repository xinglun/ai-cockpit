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

現在の immutable release baseline は `v0.2.46` です。失敗した `v0.2.35` tag は workflow run `33162800569` による公開失敗履歴として保持し、公開 Release はありません。先行する失敗 `v0.2.34` tag（workflow run `33155382717`）も保持します。失敗した staged `v0.2.32` tag は WI-299 の finalize binding defect による公開失敗履歴として保持し、installation baseline にはしません。immutable な `v0.2.30` tag も clean-batch の route defect による公開失敗履歴として保持します。
未公開の `v0.2.36` tag も staged acceptance failure の immutable な履歴として保持し、installation baseline にはしません。
以前の公開 `v0.2.45` Release は historical evidence として保持し、現在の baseline に置き換えます。

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

homebrew-handoff.json ──► external tap review（maintained tap がある場合）
                          （この repository の Runtime authority の外側）
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

未 attach の release-build checkout は `.ai` を持たない場合がありますが、この self-governed checkout は
repository-local `.ai/` を意図的に持ちます。`cockpit.toml` は `.ai/` 配下の TOML のままで、distribution が
JSON へ移行させることもありません。

## アップグレード境界

Runtime-only upgrade は共有 executable だけを置き換え、repository の `.ai/` bytes、
Contract、evidence、Work Item、knowledge を変更しない。Repository migration は別の
操作であり、新 Runtime が `MIGRATION_REQUIRED` を返したときに、明示的にレビューされ
た versioned operation として実行する。migration receipt は前後の repository digest と
Runtime version/digest に結び付き、過去の evidence は書き換えない。

N-1 acceptance harness は旧・新の公開 archive でこの境界を検証します。これは公開後 artifact
であり、source build fallback や Release truth の代替ではありません。

Release tag では publication と handoff が完了した後だけ harness を起動し、workflow は直前の
published Release を解決して receipt を独立に upload します。manual dispatch には公開済みの
`from_tag` と `to_tag` の明示入力が必要で、Release を publish しません。直前の Release がない
場合は checksum 付きの `not_applicable` result を記録します。
同じ schema の patch upgrade でも harness を実行して `migrationState: not_required` を記録し、
schema が変わる pair だけが approval-gated migration branch に進みます。

## Trust boundary

- `cockpit-release` と release workflow は local release contract、deterministic manifest、Formula projection、
  hosted check、published Release identity を扱います。
- 現在の immutable public baseline は `v0.2.46` で、public adopter acceptance と N-1 upgrade 受入れは post-release evidence です。
  external Homebrew tap は別の provider surface であり、この repository が自動的に保証するものではありません。
- 予約済みの `v0.2.24` tag と immutable な `v0.2.25` tag は公開前 failure history として保持し、公開 Release として扱わず、再利用しません。
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
3. [Reference source parity](../reference/reference-parity.ja.md) — reference template との差分。

## 技術的な深さ

Rust `cockpit-release` package は manifest、archive、Formula、handoff を strict に検証します。GitHub Actions
は五つの retained target を build し、source、verification、attestation、publication、handoff の権限を分離し、
external tap mutation を default repository token の外側に置きます。
